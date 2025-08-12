pub mod graphics_2d;

use ruin_canvas::Canvas;
use ruin_ecs::{physics_2d::PhysicsWorld, world::World};
use winit::event::WindowEvent;

use ruin_assets::{Handle, ImageTexture};

pub struct CameraInfo {
    pub zoom: f32,
    pub position: [f32; 3],
}

pub trait Graphics {
    fn render(&mut self, world: &World, canvas: &Canvas, physics: &PhysicsWorld);
    fn resize(&mut self, width: u32, height: u32);
    fn process_camera_event(&mut self, event: &WindowEvent);
    fn set_background(&mut self, color: wgpu::Color);
    fn update_camera(&mut self);
    fn load_texture_from_path(&mut self, id: &str, path: &str) -> Handle<ImageTexture>;
    fn get_camera_info(&self) -> CameraInfo;
    fn move_camera_for_follow(
        &mut self,
        dt: f32,
        position: [f32; 3],
        velocity: [f32; 3],
        acceleration: [f32; 3],
        offset: [f32; 3],
    );
}
