#[derive(Debug, Copy, Clone)]
pub enum FwdSkipMode {
    DoNothing,
    Linear,
    Nearest,
    Zeroed,
}

#[derive(Debug, Copy, Clone)]
pub struct Options {
    pub(crate) t_start: u64,
    pub(crate) t_step: u64,
    pub(crate) t_total: u64,
    pub(crate) preallocate: bool,
    pub(crate) ignore_hash: bool,
    pub(crate) max_fwd_skip: u64,
    pub(crate) fwd_skip_mode: FwdSkipMode,
}

impl Options {
    pub fn new(start_time: u64, time_step: u64, total_time: u64) -> Self {
        Self {
            t_start: start_time,
            t_step: time_step,
            t_total: total_time,
            preallocate: false,
            ignore_hash: false,
            max_fwd_skip: 0,
            fwd_skip_mode: FwdSkipMode::Nearest,
        }
    }

    pub fn preallocate(self, val: bool) -> Self {
        Self {
            preallocate: val,
            ..self
        }
    }

    pub fn ignore_hash(self, val: bool) -> Self {
        Self {
            ignore_hash: val,
            ..self
        }
    }

    pub fn max_fwd_skip(self, val: u64) -> Self {
        Self {
            max_fwd_skip: val,
            ..self
        }
    }

    pub fn fwd_skip_mode(self, val: FwdSkipMode) -> Self {
        Self {
            fwd_skip_mode: val,
            ..self
        }
    }
}
