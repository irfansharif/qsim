extern crate rand;

use rand::distributions::{Exp, IndependentSample};

// The underlying RNG, if configured (consider λ in an exponentially distributed generator for
// e.g.), should map to an events/s parameter. next_event returns an u32 integer corresponding to
// how many discrete time units of the specified resolution (1e6 for a µs scale for e.g.) would
// need to pass until the next such event.
// If the resolution is too course (1 for e.g.  corresponding to a second resolution) the return
// value might be 0, this just means we've lost potentially useful information due to rounding up
// errors.  If the next event was to occur after 5ms, a specified resolution of a second scale
// asking for the next second the event would occur would return 0 -- hardly useful information.
trait EventGenerator {
    fn next_event(&self, resolution: f64) -> u32;
}

// ExponentialGenerator generates random numbers that fit an exponential
// distribution.
pub struct ExponentialGenerator {
    exp: Exp,
}

impl ExponentialGenerator {
    fn new(lambda: f64) -> ExponentialGenerator {
        ExponentialGenerator { exp: Exp::new(lambda) }
    }
}

impl EventGenerator for ExponentialGenerator {
    fn next_event(&self, resolution: f64) -> u32 {
        (self.exp.ind_sample(&mut rand::thread_rng()) * resolution) as u32
    }
}

// Packet holds the value of the time in ticks that it was generated at.
struct Packet(u32);

// PacketGenerator generates packets according to a provided EventGenerator.
struct PacketGenerator<'a> {
    // Last tick that a packet was generated.
    last_gen: u32,
    // Ticks till next packet is generated.
    till_next: u32,
    tg: &'a EventGenerator,
}

impl<'a> PacketGenerator<'a> {
    fn new(tg: &EventGenerator) -> PacketGenerator {
        PacketGenerator {
            last_gen: 0,
            till_next: 0,
            tg: tg,
        }
    }

    // next_packet takes the current time and uses it to determine whether to generate a packet or not.
    // If a packet is generated, the last_gen time and till_next times are both updated.
    fn next_packet(&mut self, time: u32) -> Option<Packet> {
        if time - self.last_gen == self.till_next {
            self.last_gen = time;
            self.till_next = self.tg.next_event(1e6);
            Some(Packet(time))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Use `cargo test -- --nocapture` to verify the generation of exponentially distributed random
    // u32 integers, at 100 packets/s and a µs scale resolution, a typical generation would be
    // [8728, 12561, 4670, 5370, 9221].
    #[test]
    fn generate_exponential_events() {
        let eg = ExponentialGenerator::new(100.0);
        let mut events = vec![];
        for _ in 0..5 {
            events.push(eg.next_event(1e6));
        }
        println!("vals: {:?}", events)
    }

    struct TestGenerator(u32);

    impl EventGenerator for TestGenerator {
        fn next_event(&self, _: f64) -> u32 {
            self.0
        }
    }

    #[test]
    fn test_packet_generator() {
        let tg = TestGenerator(5);
        let mut pg = PacketGenerator::new(&tg);
        let mut packet = pg.next_packet(0);
        assert_eq!(packet.expect("invalid value").0, 0);
        assert_eq!(pg.last_gen, 0);
        assert_eq!(pg.till_next, 5);
        packet = pg.next_packet(3);
        assert!(packet.is_none());
        assert_eq!(pg.last_gen, 0);
        assert_eq!(pg.till_next, 5);
    }
}
