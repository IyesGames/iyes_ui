use bevy::prelude::*;
use bevy::utils::HashMap;

use crate::UiDisabled;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct ClickHandlerSet;

enum ClickBehaviorKind {
    System(bool, Option<Box<dyn System<In = (), Out = ()>>>),
    EntitySystem(bool, Option<Box<dyn System<In = Entity, Out = ()>>>),
    #[cfg(feature = "iyes_cli")]
    Cli(String),
}

/// Add this component to an interactable UI node, to specify things to run automatically on click.
///
/// You can add a sequence of any number of actions.
#[derive(Component, Default)]
pub struct OnClick {
    actions: Vec<ClickBehaviorKind>,
}

impl OnClick {
    pub fn new() -> OnClick {
        OnClick::default()
    }
    /// Run a system when the UI node is clicked
    pub fn system<S, Param>(mut self, system: S) -> OnClick
    where
        S: IntoSystem<(), (), Param>,
    {
        self.actions.push(ClickBehaviorKind::System(
            false,
            Some(Box::new(IntoSystem::into_system(system))),
        ));
        self
    }
    /// Run an "entity system" when the UI node is clicked
    ///
    /// Your system will be provided the `Entity` ID of the entity that was clicked,
    /// allowing you to query for additional data on the current entity.
    pub fn entity_system<S, Param>(mut self, system: S) -> OnClick
    where
        S: IntoSystem<Entity, (), Param>,
    {
        self.actions.push(ClickBehaviorKind::EntitySystem(
            false,
            Some(Box::new(IntoSystem::into_system(system))),
        ));
        self
    }

    /// Run an `iyes_cli` console command
    #[cfg(feature = "iyes_cli")]
    pub fn cli(mut self, cli: &str) -> OnClick {
        self.actions.push(ClickBehaviorKind::Cli(cli.to_owned()));
        self
    }
}

pub(crate) fn onclick_run_behaviors(
    world: &mut World,
    query: &mut QueryState<
        (Entity, &Interaction, &mut OnClick),
        (Changed<Interaction>, Without<UiDisabled>),
    >,
) {
    let mut behaviors: HashMap<Entity, Vec<ClickBehaviorKind>> = Default::default();
    for (entity, interaction, mut behavior) in query.iter_mut(world) {
        if *interaction == Interaction::Pressed {
            let behavior = behavior.bypass_change_detection();
            behaviors.insert(entity, core::mem::take(&mut behavior.actions));
        }
    }
    for (entity, mut actions) in behaviors {
        for action in &mut actions {
            match action {
                #[cfg(feature = "iyes_cli")]
                ClickBehaviorKind::Cli(cli) => {
                    use iyes_cli::prelude::*;
                    world.run_clicommand(cli);
                }
                ClickBehaviorKind::System(initted, system_opt) => {
                    if let Some(mut system) = system_opt.take() {
                        if !*initted {
                            system.initialize(world);
                            *initted = true;
                        }
                        system.run((), world);
                        system.apply_deferred(world);
                        *system_opt = Some(system);
                    }
                }
                ClickBehaviorKind::EntitySystem(initted, system_opt) => {
                    if let Some(mut system) = system_opt.take() {
                        if !*initted {
                            system.initialize(world);
                            *initted = true;
                        }
                        system.run(entity, world);
                        system.apply_deferred(world);
                        *system_opt = Some(system);
                    }
                }
            }
        }
        let Some(mut entity_mut) = world.get_entity_mut(entity) else { continue; };
        let Some(mut behavior_mut) = entity_mut.get_mut::<OnClick>() else { continue; };
        behavior_mut.bypass_change_detection().actions = actions;
    }
}
