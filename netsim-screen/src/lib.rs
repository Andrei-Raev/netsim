//! Минимальная визуализация симуляции: сетка мира, слои полей и агенты.
//!
//! Дизайн: только 2D‑картинка, без сложного UI. Переключение слоя — Space.

pub mod config;

use anyhow::Result;
use pixels::{Pixels, SurfaceTexture};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use netsim_core::{
    AgentSoA, CpuWorldGenerator, Event as SimEvent, EventKind, ScenarioConfig, SimPipeline,
    SimResult, WorldGrid, WorldGridGenerator,
};

pub use config::WindowConfig;

const GRID_COLOR: [u8; 4] = [0x55, 0x55, 0x55, 0xFF];
const BACKGROUND_COLOR: [u8; 4] = [0x10, 0x10, 0x10, 0xFF];
const AGENT_COLOR: [u8; 4] = [0xFF, 0x3B, 0x2F, 0xFF];
const LINK_COLOR: [u8; 4] = [0x2F, 0xC6, 0xFF, 0xFF];
const AGENT_RADIUS: i32 = 3;
const LINK_TTL_FRAMES: u64 = 6;

/// Набор доступных визуальных слоёв.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualLayer {
    Load,
    Noise,
    Bandwidth,
    Cost,
}

impl VisualLayer {
    /// Циклически переключает слой вперёд.
    pub fn next(self) -> Self {
        match self {
            Self::Load => Self::Noise,
            Self::Noise => Self::Bandwidth,
            Self::Bandwidth => Self::Cost,
            Self::Cost => Self::Load,
        }
    }

    /// Возвращает имя слоя (для отладки).
    pub fn name(self) -> &'static str {
        match self {
            Self::Load => "load",
            Self::Noise => "noise",
            Self::Bandwidth => "bandwidth",
            Self::Cost => "cost",
        }
    }
}

/// Визуализатор симуляции.
#[derive(Debug)]
pub struct SimVisualizer {
    event_loop: EventLoop<()>,
    window: winit::window::Window,
    pixels: Pixels,
    config: WindowConfig,
    layer: VisualLayer,
    scenario: ScenarioConfig,
    pipeline: SimPipeline,
    generator: CpuWorldGenerator,
    ticks_left: u64,
    last_events: Vec<SimEvent>,
    link_history: Vec<LinkTrace>,
}

#[derive(Debug, Clone, Copy)]
struct LinkTrace {
    src_index: usize,
    dst_index: usize,
    expires_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoopAction {
    None,
    Close,
}

#[derive(Debug, Clone)]
struct LoopSignals {
    action: LoopAction,
    resized: Option<winit::dpi::PhysicalSize<u32>>,
    redraw_requested: bool,
}

impl Default for LoopSignals {
    fn default() -> Self {
        Self {
            action: LoopAction::None,
            resized: None,
            redraw_requested: false,
        }
    }
}

impl SimVisualizer {
    /// Создаёт визуализатор и готовит сценарий.
    pub fn new(config: WindowConfig, scenario: ScenarioConfig) -> Result<Self> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Netsim Visualizer")
            .with_inner_size(LogicalSize::new(config.width_px, config.height_px))
            .build(&event_loop)?;

        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(config.width_px, config.height_px, surface_texture)?;

        let mut pipeline = SimPipeline::from_scenario(&scenario);
        let scene = scenario.build_scene();
        let generator = scene.into_generator();
        pipeline.set_world_grid(generator.build_grid(0));
        let ticks_left = scenario.ticks;

        Ok(Self {
            event_loop,
            window,
            pixels,
            config,
            layer: VisualLayer::Noise,
            scenario,
            pipeline,
            generator,
            ticks_left,
            last_events: Vec::new(),
            link_history: Vec::new(),
        })
    }

    /// Запускает event loop визуализации.
    pub fn run(mut self) -> Result<SimResult> {
        let mut result = self.sim_result();
        let mut running = true;

        while running {
            let signals = run_loop_iteration(&mut self.event_loop);

            if let Some(size) = signals.resized {
                let _ = self.pixels.resize_surface(size.width, size.height);
            }

            if signals.redraw_requested {
                if let Err(err) = self.render() {
                    eprintln!("Render error: {err}");
                    running = false;
                }
            }

            if self.ticks_left > 0 {
                let tick = self.pipeline.event_queue.current_tick();
                self.last_events = self.pipeline.event_queue.peek_current().to_vec();
                self.pipeline
                    .step_with_scenario(&self.scenario, &self.generator);
                self.ticks_left = self.ticks_left.saturating_sub(1);
                self.update_link_history(tick);
                self.window.request_redraw();
            } else {
                result = self.sim_result();
            }

            if signals.action == LoopAction::Close {
                running = false;
            }
        }

        Ok(result)
    }

    fn sim_result(&self) -> SimResult {
        let processed = self.scenario.ticks.saturating_sub(self.ticks_left);
        SimResult {
            ticks_processed: processed,
            stats: self.pipeline.stats.clone(),
        }
    }

    fn update_link_history(&mut self, tick: u64) {
        self.link_history.retain(|trace| trace.expires_at > tick);

        for event in &self.last_events {
            if event.kind != EventKind::Packet {
                continue;
            }
            let src_index = match self
                .pipeline
                .agents
                .agent_id
                .iter()
                .position(|id| *id == event.payload.src_id)
            {
                Some(index) => index,
                None => continue,
            };
            let dst_index = match self
                .pipeline
                .agents
                .agent_id
                .iter()
                .position(|id| *id == event.payload.dst_id)
            {
                Some(index) => index,
                None => continue,
            };

            self.link_history.push(LinkTrace {
                src_index,
                dst_index,
                expires_at: tick.saturating_add(LINK_TTL_FRAMES),
            });
        }
    }

    fn render(&mut self) -> Result<()> {
        let width = self.config.width_px;
        let height = self.config.height_px;
        let layer = self.layer;

        let Some(grid) = self.pipeline.world_grid.as_ref() else {
            self.pixels.render()?;
            return Ok(());
        };
        let agents = &self.pipeline.agents;
        let links = &self.link_history;

        let frame = self.pixels.frame_mut();
        fill_color(frame, BACKGROUND_COLOR);

        render_layer(frame, width, height, grid, layer);
        render_grid_lines(frame, width, height, grid);
        render_links(frame, width, height, grid, agents, links);
        render_agents(frame, width, height, grid, agents);

        self.pixels.render()?;
        Ok(())
    }
}

fn run_loop_iteration(event_loop: &mut EventLoop<()>) -> LoopSignals {
    let mut signals = LoopSignals::default();

    event_loop.run_return(|event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    signals.action = LoopAction::Close;
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    signals.resized = Some(size);
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Pressed {
                        if let Some(code) = input.virtual_keycode {
                            match code {
                                VirtualKeyCode::Space => {
                                    signals.redraw_requested = true;
                                }
                                VirtualKeyCode::Escape => {
                                    signals.action = LoopAction::Close;
                                    *control_flow = ControlFlow::Exit;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                signals.redraw_requested = true;
            }
            Event::MainEventsCleared => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });

    signals
}

fn fill_color(frame: &mut [u8], color: [u8; 4]) {
    for pixel in frame.chunks_exact_mut(4) {
        pixel.copy_from_slice(&color);
    }
}

fn render_grid_lines(frame: &mut [u8], width: u32, height: u32, grid: &WorldGrid) {
    if grid.width == 0 || grid.height == 0 {
        return;
    }

    for x in 0..=grid.width {
        let screen_x = ((x as f32 / grid.width as f32) * width as f32) as i32;
        for y in 0..height as i32 {
            set_pixel(frame, width, height, screen_x, y, GRID_COLOR);
        }
    }

    for y in 0..=grid.height {
        let screen_y = ((y as f32 / grid.height as f32) * height as f32) as i32;
        for x in 0..width as i32 {
            set_pixel(frame, width, height, x, screen_y, GRID_COLOR);
        }
    }
}

fn render_agents(frame: &mut [u8], width: u32, height: u32, grid: &WorldGrid, agents: &AgentSoA) {
    for index in 0..agents.len() {
        if !agents.alive[index] {
            continue;
        }
        let pos_x = agents.pos_x[index];
        let pos_y = agents.pos_y[index];

        if let Some((sx, sy)) = world_to_screen(grid, width, height, pos_x, pos_y) {
            draw_filled_circle(frame, width, height, sx, sy, AGENT_RADIUS, AGENT_COLOR);
        }
    }
}

fn render_layer(frame: &mut [u8], width: u32, height: u32, grid: &WorldGrid, layer: VisualLayer) {
    if grid.width == 0 || grid.height == 0 {
        return;
    }

    let (min_value, max_value) = grid_layer_range(grid, layer);
    let span = (max_value - min_value).max(1e-6);

    for y in 0..grid.height {
        for x in 0..grid.width {
            let Some(cell) = grid.cell(x, y) else {
                continue;
            };

            let value = match layer {
                VisualLayer::Load => cell.load,
                VisualLayer::Noise => cell.noise,
                VisualLayer::Bandwidth => cell.bandwidth,
                VisualLayer::Cost => cell.cost,
            };

            let norm = ((value - min_value) / span).clamp(0.0, 1.0);
            let color = layer_color(layer, norm);

            fill_cell(frame, width, height, grid, x, y, color);
        }
    }
}

fn render_links(
    frame: &mut [u8],
    width: u32,
    height: u32,
    grid: &WorldGrid,
    agents: &AgentSoA,
    links: &[LinkTrace],
) {
    for trace in links {
        if trace.src_index >= agents.len() || trace.dst_index >= agents.len() {
            continue;
        }
        if !agents.alive[trace.src_index] || !agents.alive[trace.dst_index] {
            continue;
        }
        let src_x = agents.pos_x[trace.src_index];
        let src_y = agents.pos_y[trace.src_index];
        let dst_x = agents.pos_x[trace.dst_index];
        let dst_y = agents.pos_y[trace.dst_index];

        let Some((sx, sy)) = world_to_screen(grid, width, height, src_x, src_y) else {
            continue;
        };
        let Some((dx, dy)) = world_to_screen(grid, width, height, dst_x, dst_y) else {
            continue;
        };

        draw_line(frame, width, height, sx, sy, dx, dy, LINK_COLOR);
    }
}

fn set_pixel(frame: &mut [u8], width: u32, height: u32, x: i32, y: i32, color: [u8; 4]) {
    if x < 0 || y < 0 {
        return;
    }
    let x = x as u32;
    let y = y as u32;
    if x >= width || y >= height {
        return;
    }
    let index = (y * width + x) as usize * 4;
    if index + 3 < frame.len() {
        frame[index..index + 4].copy_from_slice(&color);
    }
}

fn draw_filled_circle(
    frame: &mut [u8],
    width: u32,
    height: u32,
    center_x: i32,
    center_y: i32,
    radius: i32,
    color: [u8; 4],
) {
    let radius_sq = radius * radius;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= radius_sq {
                set_pixel(frame, width, height, center_x + dx, center_y + dy, color);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_line(
    frame: &mut [u8],
    width: u32,
    height: u32,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: [u8; 4],
) {
    let mut x0 = x0;
    let mut y0 = y0;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        set_pixel(frame, width, height, x0, y0, color);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn world_to_screen(
    grid: &WorldGrid,
    width: u32,
    height: u32,
    world_x: f32,
    world_y: f32,
) -> Option<(i32, i32)> {
    if grid.width == 0 || grid.height == 0 {
        return None;
    }
    if world_x < 0.0 || world_y < 0.0 {
        return None;
    }
    let max_x = grid.width as f32 * grid.cell_size;
    let max_y = grid.height as f32 * grid.cell_size;
    if world_x > max_x || world_y > max_y {
        return None;
    }

    let screen_x = (world_x / max_x * width as f32) as i32;
    let screen_y = (world_y / max_y * height as f32) as i32;
    Some((screen_x, screen_y))
}

fn fill_cell(
    frame: &mut [u8],
    width: u32,
    height: u32,
    grid: &WorldGrid,
    cell_x: usize,
    cell_y: usize,
    color: [u8; 4],
) {
    let cell_width = (width as f32 / grid.width as f32).ceil() as i32;
    let cell_height = (height as f32 / grid.height as f32).ceil() as i32;

    let start_x = (cell_x as i32 * cell_width).max(0);
    let start_y = (cell_y as i32 * cell_height).max(0);
    let end_x = (start_x + cell_width).min(width as i32);
    let end_y = (start_y + cell_height).min(height as i32);

    for y in start_y..end_y {
        for x in start_x..end_x {
            set_pixel(frame, width, height, x, y, color);
        }
    }
}

fn grid_layer_range(grid: &WorldGrid, layer: VisualLayer) -> (f32, f32) {
    let mut min_value = f32::MAX;
    let mut max_value = f32::MIN;

    for cell in &grid.cells {
        let value = match layer {
            VisualLayer::Load => cell.load,
            VisualLayer::Noise => cell.noise,
            VisualLayer::Bandwidth => cell.bandwidth,
            VisualLayer::Cost => cell.cost,
        };
        min_value = min_value.min(value);
        max_value = max_value.max(value);
    }

    if min_value == f32::MAX || max_value == f32::MIN {
        (0.0, 1.0)
    } else if (max_value - min_value).abs() < f32::EPSILON {
        (min_value, min_value + 1.0)
    } else {
        (min_value, max_value)
    }
}

fn layer_color(layer: VisualLayer, normalized: f32) -> [u8; 4] {
    let value = (normalized * 255.0) as u8;
    match layer {
        VisualLayer::Noise => [value, 0x20, 0x20, 0xA0],
        VisualLayer::Load => [0x20, value, 0x20, 0xA0],
        VisualLayer::Bandwidth => [0x20, 0x20, value, 0xA0],
        VisualLayer::Cost => [value, value, 0x20, 0xA0],
    }
}
