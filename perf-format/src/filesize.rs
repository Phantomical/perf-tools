use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Scale {
    B,
    KiB,
    MiB,
    GiB,
    TiB,
    PiB,
    EiB
}

impl Scale {
    fn factor(self) -> u64 {
        match self {
            Self::B => 1,
            Self::KiB => 1_000,
            Self::MiB => 1_000_000,
            Self::GiB => 1_000_000_000,
            Self::TiB => 1_000_000_000_000,
            Self::PiB => 1_000_000_000_000_000,
            Self::EiB => 1_000_000_000_000_000_000
        }
    }

    fn cutoff(self) -> u64 {
        self.factor() * 1000
    }
}

pub struct FileSize {
    count: u64,
    frac: Option<u64>,
    scale: Scale
}

impl FileSize {
    pub fn new(size: u64) -> Self {
        use self::Scale::*;

        let class = match size {
            x if x <   B.cutoff() =>   B,
            x if x < KiB.cutoff() => KiB,
            x if x < MiB.cutoff() => MiB,
            x if x < GiB.cutoff() => GiB,
            x if x < TiB.cutoff() => TiB,
            x if x < PiB.cutoff() => PiB,
            _ => EiB
        };
        

        if class == B {
            return FileSize {
                count: size,
                frac: None,
                scale: B
            };
        }

        let divisor = class.factor();
        let count = size / divisor;
        let frac = size / (divisor / 100);

        FileSize {
            count,
            frac: Some(frac),
            scale: class
        }
    }
}

impl fmt::Display for FileSize {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if let Some(frac) = self.frac {
            write!(fmt, "{}.{:02} {:?}", self.count, frac, self.scale)
        } else {
            write!(fmt, "{} B", self.count)
        }
    }
}
