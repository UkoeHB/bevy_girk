//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use iyes_progress::prelude::Progress;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub(crate) struct InitializationProgressCache
{
    prev_progress : Progress,
    progress      : Progress
}

impl Default for InitializationProgressCache
{
    fn default() -> InitializationProgressCache
    {
        // values must be different at the start so progress_changed_last_update() will be true on construction
        InitializationProgressCache{
                prev_progress : Progress{ done: 1, total: 0 },
                progress      : Progress::default()
            }
    }
}

impl InitializationProgressCache
{
    pub(crate) fn set_progress(&mut self, progress: Progress)
    {
        self.prev_progress = self.progress;
        self.progress      = progress;
    }

    pub(crate) fn set_progress_complete(&mut self)
    {
        self.set_progress(Progress{ done: 1, total: 1 });
    }

    pub(crate) fn progress_changed_last_update(&self) -> bool
    {
        if self.progress.done  != self.prev_progress.done  { return true; }
        if self.progress.total != self.prev_progress.total { return true; }
        return false;
    }

    pub(crate) fn progress(&self) -> Progress { self.progress }
}

//-------------------------------------------------------------------------------------------------------------------
