use std::io::Cursor;

use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_replicon::{core::{ctx::WriteCtx, replication_registry::{command_fns::default_remove, rule_fns::RuleFns}}, prelude::*};
use bevy_replicon_repair::*;
use serde::{de::DeserializeOwned, Serialize};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn update_react_component<C: ReactComponent>(
    In((component, entity)): In<(C, Entity)>,
    mut c: Commands,
    mut query: Query<&mut React<C>>,
)
{
    let Ok(mut existing) = query.get_mut(entity) else {
        c.react().insert(entity, component);
        return;
    };
    *existing.get_mut(&mut c) = component;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn write_react_component <C: Component + ReactComponent + DeserializeOwned>(
    ctx: &mut WriteCtx,
    rule_fns: &RuleFns<C>,
    entity: &mut EntityMut,
    cursor: &mut Cursor<&[u8]>,
) -> bincode::Result<()>
{
    let component: C = rule_fns.deserialize(ctx, cursor)?;
    let entity_id = entity.id();
    ctx.commands.add(move |world: &mut World| syscall(world, (component, entity_id), update_react_component));

    Ok(())
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Extension trait for combining `bevy_replicon`, `bevy_replicon_repair`, and `bevy_cobweb`.
pub trait ReplicateRepairReactExt
{
    /// Replicate-repair a [`ReactComponent`].
    ///
    /// The component `C` should be a plain [`Component`] on the server, and then it will be replicated as a
    /// [`ReactComponent`] on the client.
    fn replicate_repair_react<C>(&mut self) -> &mut Self
    where
        C: Component + ReactComponent + Serialize + DeserializeOwned;
}

impl ReplicateRepairReactExt for App
{
    fn replicate_repair_react<C>(&mut self) -> &mut Self
    where
        C: Component + ReactComponent + Serialize + DeserializeOwned
    {
        self.set_command_fns::<C>(
                write_react_component::<C>,
                default_remove::<React<C>>,
            )
            .replicate_repair_with::<C>(
                RuleFns::<C>::default(),
                repair_component::<React<C>>,
            )
    }
}

//-------------------------------------------------------------------------------------------------------------------
