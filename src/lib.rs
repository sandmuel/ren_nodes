#![feature(specialization)]

use necs::{NodeBuilder, NodeId, NodeRef, NodeTrait};
use std::ops::{Deref, DerefMut};

/// Nodes with this trait have [`process`] called on them each frame.
pub trait Process: NodeTrait {
    fn process(&mut self, world: &World);
}

#[derive(Default, Debug)]
pub struct World {
    world: necs::World,
    process_nodes: Vec<NodeId>,
}

impl Deref for World {
    type Target = necs::World;
    fn deref(&self) -> &Self::Target {
        &self.world
    }
}

impl DerefMut for World {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.world
    }
}

impl World {
    pub fn spawn<T: NodeBuilder + 'static>(&mut self, node: T) -> NodeId {
        Spawn::spawn(node, self)
    }

    pub fn register_node<T>(&mut self)
    where
        T: NodeRef,
    {
        self.world.register_node::<T>();
        T::register(self);
    }

    pub fn process_nodes(&self) {
        self.process_nodes.iter().for_each(|id| {
            let mut node = self.world.get_node_resilient::<dyn Process>(*id);
            node.process(self);
        })
    }
}

trait Spawn {
    fn spawn(self, world: &mut World) -> NodeId;
}

impl<T: NodeBuilder + 'static> Spawn for T {
    // If T only implements NodeBuilder, then just spawn it.
    default fn spawn(self, world: &mut World) -> NodeId {
        world.spawn_node(self)
    }
}

impl<T: NodeBuilder<AsNodeRef: Process> + 'static> Spawn for T {
    // If T also implements Process, then spawn it and add it to the list of
    // processed nodes.
    fn spawn(self, world: &mut World) -> NodeId {
        let node_id = world.spawn_node(self);
        world.process_nodes.push(node_id);
        node_id
    }
}

#[doc(hidden)]
trait ProcessRegister {
    fn register(nodes: &mut World);
}

impl<T: NodeRef> ProcessRegister for T {
    // If T only meets the NodeRef trait bound, then do nothing.
    default fn register(_: &mut World) {}
}

impl<T: NodeRef<Instance<'static>: Process> + Process> ProcessRegister for T {
    // If T also meets the Process trait bound, then register it.
    fn register(world: &mut World) {
        world.register_trait::<T, dyn Process, _>(|x| Box::new(x));
    }
}
