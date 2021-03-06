use msp430g2211;

macro_rules! set_bits_with_mask {
    ($r:ident, $w:ident, $m:expr) => { $w.bits($r.bits() | $m) };
}

macro_rules! clear_bits_with_mask {
    ($r:ident, $w:ident, $m:expr) => { $w.bits($r.bits() & !$m) };
}

pub struct KeyboardPins {
    pub at_clk : Pin,
    pub at_data : Pin,
    pub xt_clk : Pin,
    pub xt_data : Pin,
    pub xt_sense : Pin,
    // was_initialized : bool
}

impl KeyboardPins {
    // Safe as long as only one copy exists in memory (and it doesn't make sense for two copies to
    // exist); P1DIR can only be accessed from within this module, and never from an interrupt.
    pub const fn new() -> KeyboardPins {
        KeyboardPins {
            at_clk : Pin::new(0),
            at_data : Pin::new(4),
            xt_clk : Pin::new(2),
            xt_data : Pin::new(3),
            xt_sense : Pin::new(1)
        }
    }

    // Not safe in the general case, but in my code base, I only call this once during
    // initialization before the only interrupts that touches these registers is enabled.
    // Option 1: Possible to make fully safe using was_initialized?
    // Pitfall 1: Does globally enable
    pub fn idle(&self, p : &msp430g2211::PORT_1_2)  -> () {
        p.p1dir.write(|w| w.bits(0x00));
        p.p1ifg.modify(|r, w| clear_bits_with_mask!(r, w, self.at_clk.bitmask()));
        p.p1ies.modify(|r, w| set_bits_with_mask!(r, w, self.at_clk.bitmask()));
        p.p1ie.modify(|r, w| set_bits_with_mask!(r, w, self.at_clk.bitmask()));
    }

    pub fn disable_at_clk_int(&self, p : &msp430g2211::PORT_1_2) -> () {
        p.p1ie.modify(|r, w| clear_bits_with_mask!(r, w, self.at_clk.bitmask()));
    }

    // Unsafe because can be used in contexts where it's assumed pin ints can't occur.
    pub unsafe fn enable_at_clk_int(&self, p : &msp430g2211::PORT_1_2) -> () {
        p.p1ie.modify(|r, w| set_bits_with_mask!(r, w, self.at_clk.bitmask()));
    }

    pub fn clear_at_clk_int(&self, p : &msp430g2211::PORT_1_2) -> () {
        p.p1ifg.modify(|r, w| clear_bits_with_mask!(r, w, self.at_clk.bitmask()));
    }

    pub fn at_idle(&self, p : &msp430g2211::PORT_1_2) -> () {
        // XXX: Mutable borrow happens twice if we borrow port first and then call these
        // fns?
        self.at_clk.set(p);
        self.at_data.set(p);
        {

            let at_mask : u8 = self.at_clk.bitmask() | self.at_data.bitmask();
            p.p1dir.modify(|r, w| clear_bits_with_mask!(r, w, at_mask));
        }
    }

    pub fn at_inhibit(&self, p : &msp430g2211::PORT_1_2) -> () {
        // XXX: Mutable borrow happens twice if we borrow port first and then call these
        // fns?
        self.at_clk.unset(p);
        self.at_data.set(p);
        {

            let at_mask : u8 = self.at_clk.bitmask() | self.at_data.bitmask();
            p.p1dir.modify(|r, w| set_bits_with_mask!(r, w, at_mask));
        }
    }

    #[allow(dead_code)]
    pub fn at_send(&self, p : &msp430g2211::PORT_1_2) -> () {
        self.at_clk.set(p);
        self.at_data.set(p);
        self.at_clk.mk_in(p);
        self.at_data.mk_out(p);
    }

    // Why in japaric's closures access to the pins for an actual write aren't wrapped in unsafe?
    pub fn xt_out(&self, p : &msp430g2211::PORT_1_2) -> () {
        let xt_mask : u8 = self.xt_clk.bitmask() | self.xt_data.bitmask();
        p.p1out.modify(|r, w| set_bits_with_mask!(r, w, xt_mask));
        p.p1dir.modify(|r, w| set_bits_with_mask!(r, w, xt_mask));
    }

    pub fn xt_in(&self, p : &msp430g2211::PORT_1_2) -> () {
        let xt_mask : u8 = self.xt_clk.bitmask() | self.xt_data.bitmask();
        p.p1out.modify(|r, w| set_bits_with_mask!(r, w, self.xt_data.bitmask()));
        p.p1dir.modify(|r, w| clear_bits_with_mask!(r, w, xt_mask));
    }
}


pub struct Pin {
    loc : u8
}

impl Pin {
    pub const fn new(pin_no : u8) -> Pin {
        Pin { loc : pin_no }
    }

    fn bitmask(&self) -> u8 {
        (1 << self.loc)
    }

    pub fn set(&self, p : &msp430g2211::PORT_1_2) -> () {
        p.p1out.modify(|r, w| set_bits_with_mask!(r, w, self.bitmask()));
    }

    pub fn unset(&self, p : &msp430g2211::PORT_1_2) -> () {
        p.p1out.modify(|r, w| clear_bits_with_mask!(r, w, self.bitmask()));
    }

    pub fn mk_in(&self, p : &msp430g2211::PORT_1_2) -> () {
        p.p1dir.modify(|r, w| clear_bits_with_mask!(r, w, self.bitmask()));
    }

    #[allow(dead_code)]
    pub fn mk_out(&self, p : &msp430g2211::PORT_1_2) -> () {
        p.p1dir.modify(|r, w| set_bits_with_mask!(r, w, self.bitmask()));
    }


    // No side effects from reading pins- these fcns are safe.
    pub fn is_set(&self, p: &msp430g2211::PORT_1_2) ->  bool {
        (p.p1in.read().bits() & self.bitmask()) != 0
    }

    pub fn is_unset(&self, p: &msp430g2211::PORT_1_2) -> bool {
        (p.p1in.read().bits() & self.bitmask()) == 0
    }
}
