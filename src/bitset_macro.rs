// https://gist.github.com/bjz/9244400

#[macro_escape];
// Needs to be in root of the crate
//#[feature(macro_rules)];
#[allow(non_camel_case_types)];

macro_rules! bitset(
    ($BitSet:ident: $T:ty {
        $($VALUE:ident = $value:expr),+
    }) => (
        #[deriving(Eq, Ord)]
        pub struct $BitSet {
            priv bits: $T,
        }

        $(pub static $VALUE: $BitSet = $BitSet { bits: $value };)+

        impl $BitSet {
            /// The empty bitset.
            pub fn empty() -> $BitSet {
                $BitSet { bits: 0 }
            }

            /// Returns `true` if the biset is empty.
            pub fn is_empty(&self) -> bool {
                *self == $BitSet::empty()
            }

            /// Returns `true` if any elements of the bitset intersect
            /// with the other bitset.
            pub fn intersects(&self, other: $BitSet) -> bool {
                !(self & other).is_empty()
            }

            /// Returns `true` if the bitset containts all the elements of the
            /// other bitset.
            pub fn contains(&self, other: $BitSet) -> bool {
                (self & other) == other
            }

            /// Inserts a set of bits in-place.
            pub fn insert(&mut self, other: $BitSet) {
                self.bits |= other.bits;
            }

            /// Removes a set of bits in-place.
            pub fn remove(&mut self, other: $BitSet) {
                self.bits &= !other.bits;
            }
        }

        impl Sub<$BitSet, $BitSet> for $BitSet {
            /// Returns the difference between the two bitsets
            fn sub(&self, other: &$BitSet) -> $BitSet {
                $BitSet { bits: self.bits & !other.bits }
            }
        }

        impl BitOr<$BitSet, $BitSet> for $BitSet {
            /// Returns the union of the two bitsets
            fn bitor(&self, other: &$BitSet) -> $BitSet {
                $BitSet { bits: self.bits | other.bits }
            }
        }

        impl BitAnd<$BitSet, $BitSet> for $BitSet {
            /// Returns the intersection between the two bitsets
            fn bitand(&self, other: &$BitSet) -> $BitSet {
                $BitSet { bits: self.bits & other.bits }
            }
        }
    )
)

/*
bitset!(SDL_Hats: u32 {
    SDL_HAT_CENTERED    = 0x00,
    SDL_HAT_UP          = 0x01,
    SDL_HAT_RIGHT       = 0x02,
    SDL_HAT_DOWN        = 0x04,
    SDL_HAT_LEFT        = 0x08,
    // No CTFE yet, so we must access the `bits` field directly in these constexprs
    SDL_HAT_RIGHTUP     = SDL_HAT_RIGHT.bits | SDL_HAT_UP.bits,
    SDL_HAT_RIGHTDOWN   = SDL_HAT_RIGHT.bits | SDL_HAT_DOWN.bits,
    SDL_HAT_LEFTUP      = SDL_HAT_LEFT.bits | SDL_HAT_UP.bits,
    SDL_HAT_LEFTDOWN    = SDL_HAT_LEFT.bits | SDL_HAT_DOWN.bits
})

fn main() {
    let x = SDL_HAT_CENTERED | SDL_HAT_UP | SDL_HAT_RIGHT;
    assert!(x.contains(SDL_HAT_UP | SDL_HAT_RIGHT));
    assert!(x.intersects(SDL_HAT_RIGHT | SDL_HAT_DOWN));
    assert_eq!(x - SDL_HAT_CENTERED, SDL_HAT_UP | SDL_HAT_RIGHT)
}
*/
