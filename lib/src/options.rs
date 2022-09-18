#[derive(Debug, Copy, Clone)]
pub enum FwdSkipMode {
    DoNothing,
    Linear,
    Nearest,
    Zeroed,
}

#[derive(Debug, Copy, Clone)]
pub struct Options {
    pub(crate) preallocate: bool,
    pub(crate) max_fwd_skip: u64,
    pub(crate) fwd_skip_mode: FwdSkipMode,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            preallocate: false,
            max_fwd_skip: 3,
            fwd_skip_mode: FwdSkipMode::Nearest,
        }
    }
}

impl Options {
    pub fn preallocate(self, val: bool) -> Self {
        Self {
            preallocate: val,
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
