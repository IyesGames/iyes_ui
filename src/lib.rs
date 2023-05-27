use bevy::prelude::*;

pub mod onclick;

pub mod prelude {
    pub use crate::UiExtrasPlugin;
    pub use crate::UiDisabled;
    pub use crate::onclick::OnClick;
}

pub struct UiExtrasPlugin;

impl Plugin for UiExtrasPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            crate::onclick::onclick_run_behaviors
                .in_set(crate::onclick::ClickHandlerSet)
        );
    }
}

/// For disabling UI elements; our systems for handling Interactions should look for this
#[derive(Component)]
pub struct UiDisabled;
