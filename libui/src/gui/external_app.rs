use masonry_winit::app::{AppDriver, MasonryUserEvent};
use winit::{
    application::ApplicationHandler,
    event::ElementState,
    keyboard::{KeyCode, PhysicalKey},
};

pub struct ExternalApp {
    pub masonry_state: masonry_winit::app::MasonryState<'static>,
    pub app_driver: Box<dyn AppDriver>,
}

impl ApplicationHandler<MasonryUserEvent> for ExternalApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.masonry_state
            .handle_resumed(event_loop, &mut *self.app_driver);
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.masonry_state.handle_suspended(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.masonry_state.handle_about_to_wait(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        self.masonry_state.handle_window_event(
            event_loop,
            window_id,
            event,
            self.app_driver.as_mut(),
        );
    }

    fn user_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: MasonryUserEvent,
    ) {
        self.masonry_state
            .handle_user_event(event_loop, event, self.app_driver.as_mut());
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        // Handle the escape key to exit the app outside of masonry/xilem
        if let winit::event::DeviceEvent::Key(key) = &event
            && key.state == ElementState::Pressed
            && key.physical_key == PhysicalKey::Code(KeyCode::Escape)
        {
            event_loop.exit();
            return;
        }

        self.masonry_state.handle_device_event(
            event_loop,
            device_id,
            event,
            self.app_driver.as_mut(),
        );
    }

    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        self.masonry_state.handle_new_events(event_loop, cause);
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.masonry_state.handle_exiting(event_loop);
    }

    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.masonry_state.handle_memory_warning(event_loop);
    }
}
