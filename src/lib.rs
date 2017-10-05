extern crate rand;

use rand::distributions::{IndependentSample, Range};

/* AvailabilityTimeGenerator generates random numbers that fit an exponential distribution */
pub struct AvailabilityTimeGenerator {
    lambda: f64,
    range: Range<f64>,
}

impl AvailabilityTimeGenerator {
    fn new(lambda: f64) -> AvailabilityTimeGenerator {
        let range: Range<f64> = Range::new(0.0, 1.0);
        AvailabilityTimeGenerator {
            lambda: lambda,
            range: range,
        }
    }

    fn generate(&self) -> u32 {
        let mut rng = rand::thread_rng();
        let u = self.range.ind_sample(&mut rng);
        // Use a tick duration of 1 microsecond like in lab manual example
        ((-1.0/self.lambda) * (1.0 - u).ln() * 1_000_000.0).trunc() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Just testing to check that the numbers generated seem to be random... use
    // cargo test -- --nocapture to see stdout
    #[test]
    fn print_random_numbers() {
        let atg = AvailabilityTimeGenerator::new(100.0);
        for i in 0..10 {
            println!("{}", atg.generate())
        }
    }
}