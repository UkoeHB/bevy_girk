//local shortcuts

//third-party shortcuts

//standard shortcuts
use std::fmt::Debug;

//-------------------------------------------------------------------------------------------------------------------

pub trait HandleReqs: enfync::Handle + Debug + Send + Sync + 'static {}

impl<S: enfync::Handle + Debug + Send + Sync + 'static> HandleReqs for S {}

//-------------------------------------------------------------------------------------------------------------------

/// Wrapper trait for `dyn Fn() -> impl enfync::Handle`.
pub trait TaskSpawnerGetterFn<S: HandleReqs>: Fn() -> S + Sync + Send
{}

impl<F, S: HandleReqs> TaskSpawnerGetterFn<S> for F
where
    F: Fn() -> S + Sync + Send,
{}

pub type TaskSpawnerGetterFnT<S> = dyn TaskSpawnerGetterFn<S, Output = S>;

impl<S: HandleReqs> std::fmt::Debug for TaskSpawnerGetterFnT<S>
{
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------------------------
