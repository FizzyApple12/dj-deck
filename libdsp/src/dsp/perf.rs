use num::{Complex, Float};

#[inline]
pub fn mul<const CONJUGATE_SECOND: bool, V>(a: &Complex<V>, b: &Complex<V>) -> Complex<V>
where
    V: Float,
{
    if CONJUGATE_SECOND {
        Complex::<V>::new(b.re * a.re + b.im * a.im, b.re * a.im - b.im * a.re)
    } else {
        Complex::<V>::new(a.re * b.re - a.im * b.im, a.re * b.im + a.im * b.re)
    }
}

// todo: enable denormal stopping
pub struct StopDenormals {
    // #[cfg(target_arch = "x86_64")]
    // control_status_register: u32,
    // #[cfg(target_arch = "arm")]
    // status: usize,
}

impl StopDenormals {
    #[allow(unused, clippy::unreadable_literal)]
    pub fn start() -> StopDenormals {
        cfg_select! {
            // target_arch = "x86_64" => {
            //     use std::arch::{asm, is_x86_feature_detected};

            //     let mut control_status_register = 0;

            //     if is_x86_feature_detected!("sse") {
            //         unsafe {
            //             asm!("stmxcsr {0:e}", out(reg) control_status_register);

            //             let new_control_status_register = control_status_register | 0x8040;

            //             asm!("ldmxcsr {0:e}", in(reg) new_control_status_register);
            //         }
            //     }

            //     StopDenormals {
            //         control_status_register,
            //     }
            // }
            // target_arch = "arm" => {
            //     use std::arch::{asm, is_arm_feature_detected};

            //     let mut status = 0;

            //     if is_arm_feature_detected!("neon") {
            //         unsafe {
            //             asm!("mrs {0}, fpcr", out(reg) status);
            //         }

            //         let new_status = status | 0x01000000;

            //         unsafe {
            //             asm!("msr fpcr, {0}", in(reg) new_status);
            //         }
            //     }

            //     StopDenormals { status }
            // }
            _ => {
                StopDenormals {}
            }
        }
    }
}

impl Drop for StopDenormals {
    fn drop(&mut self) {
        cfg_select! {
            // target_arch = "x86_64" => {
            //     use std::arch::{asm, is_x86_feature_detected};

            //     if is_x86_feature_detected!("sse") {
            //         let control_status_register = self.control_status_register;

            //         unsafe {
            //             asm!("ldmxcsr {0:e}", in(reg) control_status_register);
            //         }
            //     }
            // }
            // target_arch = "arm" => {
            //     use std::arch::{asm, is_arm_feature_detected};

            //     if is_arm_feature_detected!("neon") {
            //         let status = self.status;

            //         unsafe {
            //             asm!("msr fpcr, {0}", in(reg) status);
            //         }
            //     }
            // }
            _ => {}
        };
    }
}
