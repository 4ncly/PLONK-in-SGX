//! This module provides an implementation of the $\mathbb{G}_1$ group of BLS12-381.

use crate::fp::Fp;
use crate::BlsScalar;

use core::borrow::Borrow;
use core::iter::Sum;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use dusk_bytes::{Error as BytesError, HexDebug, Serializable};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

#[cfg(feature = "canon")]
use canonical::{Canon, CanonError, Sink, Source};
#[cfg(feature = "canon")]
use canonical_derive::Canon;
#[cfg(feature = "serde_req")]
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "rkyv-impl")]
use bytecheck::{CheckBytes, ErrorBox, StructCheckError};
#[cfg(feature = "rkyv-impl")]
use rkyv::{
    out_field, Archive, Deserialize as RkyvDeserialize, Fallible, Serialize as RkyvSerialize,
};

/// This is an element of $\mathbb{G}_1$ represented in the affine coordinate space.
/// It is ideal to keep elements in this representation to reduce memory usage and
/// improve performance through the use of mixed curve model arithmetic.
///
/// Values of `G1Affine` are guaranteed to be in the $q$-order subgroup unless an
/// "unchecked" API was misused.
#[derive(Copy, Clone, HexDebug)]
pub struct G1Affine {
    pub(crate) x: Fp,
    pub(crate) y: Fp,
    infinity: Choice,
}

#[cfg(feature = "rkyv-impl")]
#[allow(missing_docs)]
#[allow(missing_debug_implementations)]
pub struct ArchivedG1Affine {
    x: <Fp as Archive>::Archived,
    y: <Fp as Archive>::Archived,
    infinity: <u8 as Archive>::Archived,
}

#[cfg(feature = "rkyv-impl")]
impl<C> CheckBytes<C> for ArchivedG1Affine {
    type Error = StructCheckError;

    unsafe fn check_bytes<'a>(
        value: *const Self,
        context: &mut C,
    ) -> Result<&'a Self, Self::Error> {
        <<Fp as Archive>::Archived as CheckBytes<C>>::check_bytes(&(*value).x, context).map_err(
            |e| StructCheckError {
                field_name: "x",
                inner: ErrorBox::new(e),
            },
        )?;
        <<Fp as Archive>::Archived as CheckBytes<C>>::check_bytes(&(*value).y, context).map_err(
            |e| StructCheckError {
                field_name: "y",
                inner: ErrorBox::new(e),
            },
        )?;
        <<u8 as Archive>::Archived as CheckBytes<C>>::check_bytes(&(*value).infinity, context)
            .map_err(|e| StructCheckError {
                field_name: "infinity",
                inner: ErrorBox::new(e),
            })?;
        Ok(&*value)
    }
}

#[cfg(feature = "rkyv-impl")]
#[allow(missing_docs)]
#[allow(missing_debug_implementations)]
pub struct G1AffineResolver {
    x: <Fp as Archive>::Resolver,
    y: <Fp as Archive>::Resolver,
    infinity: <u8 as Archive>::Resolver,
}

#[cfg(feature = "rkyv-impl")]
impl Archive for G1Affine {
    type Archived = ArchivedG1Affine;
    type Resolver = G1AffineResolver;

    unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
        let (fp, fo) = out_field!(out.x);
        self.x.resolve(pos + fp, resolver.x, fo);

        let (fp, fo) = out_field!(out.y);
        self.y.resolve(pos + fp, resolver.y, fo);

        let (fp, fo) = out_field!(out.infinity);
        let infinity = self.infinity.unwrap_u8();
        #[allow(clippy::unit_arg)]
        infinity.resolve(pos + fp, resolver.infinity, fo);
    }
}

#[cfg(feature = "rkyv-impl")]
impl<S: Fallible + ?Sized> RkyvSerialize<S> for G1Affine {
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        let choice = self.infinity.unwrap_u8();

        Ok(Self::Resolver {
            x: <Fp as RkyvSerialize<S>>::serialize(&self.x, serializer)?,
            y: <Fp as RkyvSerialize<S>>::serialize(&self.y, serializer)?,
            infinity: <u8 as RkyvSerialize<S>>::serialize(&choice, serializer)?,
        })
    }
}

#[cfg(feature = "rkyv-impl")]
impl<D: Fallible + ?Sized> RkyvDeserialize<G1Affine, D> for ArchivedG1Affine {
    fn deserialize(&self, deserializer: &mut D) -> Result<G1Affine, D::Error> {
        let infinity = <u8 as RkyvDeserialize<u8, D>>::deserialize(&self.infinity, deserializer)?;
        let infinity = Choice::from(infinity);

        Ok(G1Affine {
            x: self.x.deserialize(deserializer)?,
            y: self.y.deserialize(deserializer)?,
            infinity,
        })
    }
}

#[cfg(feature = "canon")]
impl Canon for G1Affine {
    fn encode(&self, sink: &mut Sink) {
        sink.copy_bytes(&self.to_bytes());
    }

    fn decode(source: &mut Source) -> Result<Self, CanonError> {
        let mut bytes = [0u8; Self::SIZE];

        bytes.copy_from_slice(source.read_bytes(Self::SIZE));

        Self::from_bytes(&bytes).map_err(|_| CanonError::InvalidEncoding)
    }

    fn encoded_len(&self) -> usize {
        Self::SIZE
    }
}

impl Default for G1Affine {
    fn default() -> G1Affine {
        G1Affine::identity()
    }
}

impl<'a> From<&'a G1Projective> for G1Affine {
    fn from(p: &'a G1Projective) -> G1Affine {
        let zinv = p.z.invert().unwrap_or(Fp::zero());
        let zinv2 = zinv.square();
        let x = p.x * zinv2;
        let zinv3 = zinv2 * zinv;
        let y = p.y * zinv3;

        let tmp = G1Affine {
            x,
            y,
            infinity: Choice::from(0u8),
        };

        G1Affine::conditional_select(&tmp, &G1Affine::identity(), zinv.is_zero())
    }
}

impl From<G1Projective> for G1Affine {
    fn from(p: G1Projective) -> G1Affine {
        G1Affine::from(&p)
    }
}

impl Serializable<48> for G1Affine {
    type Error = BytesError;

    /// Serializes this element into compressed form. See [`notes::serialization`](crate::notes::serialization)
    /// for details about how group elements are serialized.
    fn to_bytes(&self) -> [u8; Self::SIZE] {
        // Strictly speaking, self.x is zero already when self.infinity is true, but
        // to guard against implementation mistakes we do not assume this.
        let mut res = Fp::conditional_select(&self.x, &Fp::zero(), self.infinity).to_bytes();

        // This point is in compressed form, so we set the most significant bit.
        res[0] |= 1u8 << 7;

        // Is this point at infinity? If so, set the second-most significant bit.
        res[0] |= u8::conditional_select(&0u8, &(1u8 << 6), self.infinity);

        // Is the y-coordinate the lexicographically largest of the two associated with the
        // x-coordinate? If so, set the third-most significant bit so long as this is not
        // the point at infinity.
        res[0] |= u8::conditional_select(
            &0u8,
            &(1u8 << 5),
            (!self.infinity) & self.y.lexicographically_largest(),
        );

        res
    }

    /// Attempts to deserialize a compressed element. See [`notes::serialization`](crate::notes::serialization)
    /// for details about how group elements are serialized.
    fn from_bytes(buf: &[u8; Self::SIZE]) -> Result<Self, Self::Error> {
        // We already know the point is on the curve because this is established
        // by the y-coordinate recovery procedure in from_compressed_unchecked().

        let compression_flag_set = Choice::from((buf[0] >> 7) & 1);
        let infinity_flag_set = Choice::from((buf[0] >> 6) & 1);
        let sort_flag_set = Choice::from((buf[0] >> 5) & 1);

        // Attempt to obtain the x-coordinate
        let x = {
            let mut tmp = [0; Self::SIZE];
            tmp.copy_from_slice(&buf[..Self::SIZE]);

            // Mask away the flag bits
            tmp[0] &= 0b0001_1111;

            Fp::from_bytes(&tmp)
        };

        let x: Option<Self> = x
            .and_then(|x| {
                // If the infinity flag is set, return the value assuming
                // the x-coordinate is zero and the sort bit is not set.
                //
                // Otherwise, return a recovered point (assuming the correct
                // y-coordinate can be found) so long as the infinity flag
                // was not set.
                CtOption::new(
                    G1Affine::identity(),
                    infinity_flag_set & // Infinity flag should be set
                compression_flag_set & // Compression flag should be set
                (!sort_flag_set) & // Sort flag should not be set
                x.is_zero(), // The x-coordinate should be zero
                )
                .or_else(|| {
                    // Recover a y-coordinate given x by y = sqrt(x^3 + 4)
                    ((x.square() * x) + B).sqrt().and_then(|y| {
                        // Switch to the correct y-coordinate if necessary.
                        let y = Fp::conditional_select(
                            &y,
                            &-y,
                            y.lexicographically_largest() ^ sort_flag_set,
                        );

                        CtOption::new(
                            G1Affine {
                                x,
                                y,
                                infinity: infinity_flag_set,
                            },
                            (!infinity_flag_set) & // Infinity flag should not be set
                        compression_flag_set, // Compression flag should be set
                        )
                    })
                })
            })
            .and_then(|p| CtOption::new(p, p.is_torsion_free()))
            .into();

        x.ok_or(BytesError::InvalidData)
    }
}

#[cfg(feature = "serde_req")]
impl Serialize for G1Affine {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;
        let mut tup = serializer.serialize_tuple(G1Affine::SIZE)?;
        for byte in self.to_bytes().iter() {
            tup.serialize_element(byte)?;
        }
        tup.end()
    }
}

#[cfg(feature = "serde_req")]
impl<'de> Deserialize<'de> for G1Affine {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct G1AffineVisitor;

        impl<'de> Visitor<'de> for G1AffineVisitor {
            type Value = G1Affine;

            fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                formatter.write_str("a 48-byte cannonical compressed G1Affine point from Bls12_381")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<G1Affine, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut bytes = [0u8; G1Affine::SIZE];
                for i in 0..G1Affine::SIZE {
                    bytes[i] = seq
                        .next_element()?
                        .ok_or(serde::de::Error::invalid_length(i, &"expected 48 bytes"))?;
                }

                G1Affine::from_bytes(&bytes).map_err(|_| {
                    serde::de::Error::custom(&"compressed G1Affine was not canonically encoded")
                })
            }
        }

        deserializer.deserialize_tuple(G1Affine::SIZE, G1AffineVisitor)
    }
}

impl ConstantTimeEq for G1Affine {
    fn ct_eq(&self, other: &Self) -> Choice {
        // The only cases in which two points are equal are
        // 1. infinity is set on both
        // 2. infinity is not set on both, and their coordinates are equal

        (self.infinity & other.infinity)
            | ((!self.infinity)
                & (!other.infinity)
                & self.x.ct_eq(&other.x)
                & self.y.ct_eq(&other.y))
    }
}

impl ConditionallySelectable for G1Affine {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        G1Affine {
            x: Fp::conditional_select(&a.x, &b.x, choice),
            y: Fp::conditional_select(&a.y, &b.y, choice),
            infinity: Choice::conditional_select(&a.infinity, &b.infinity, choice),
        }
    }
}

impl Eq for G1Affine {}
impl PartialEq for G1Affine {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        bool::from(self.ct_eq(other))
    }
}

impl<'a> Neg for &'a G1Affine {
    type Output = G1Affine;

    #[inline]
    fn neg(self) -> G1Affine {
        G1Affine {
            x: self.x,
            y: Fp::conditional_select(&-self.y, &Fp::one(), self.infinity),
            infinity: self.infinity,
        }
    }
}

impl Neg for G1Affine {
    type Output = G1Affine;

    #[inline]
    fn neg(self) -> G1Affine {
        -&self
    }
}

impl<'a, 'b> Add<&'b G1Projective> for &'a G1Affine {
    type Output = G1Projective;

    #[inline]
    fn add(self, rhs: &'b G1Projective) -> G1Projective {
        rhs.add_mixed(self)
    }
}

impl<'a, 'b> Add<&'b G1Affine> for &'a G1Projective {
    type Output = G1Projective;

    #[inline]
    fn add(self, rhs: &'b G1Affine) -> G1Projective {
        self.add_mixed(rhs)
    }
}

impl<'a, 'b> Sub<&'b G1Projective> for &'a G1Affine {
    type Output = G1Projective;

    #[inline]
    fn sub(self, rhs: &'b G1Projective) -> G1Projective {
        self + (-rhs)
    }
}

impl<'a, 'b> Sub<&'b G1Affine> for &'a G1Projective {
    type Output = G1Projective;

    #[inline]
    fn sub(self, rhs: &'b G1Affine) -> G1Projective {
        self + (-rhs)
    }
}

impl<T> Sum<T> for G1Projective
where
    T: Borrow<G1Projective>,
{
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = T>,
    {
        iter.fold(Self::identity(), |acc, item| acc + item.borrow())
    }
}

impl_binops_additive!(G1Projective, G1Affine);
impl_binops_additive_specify_output!(G1Affine, G1Projective, G1Projective);

const B: Fp = Fp::from_raw_unchecked([
    0xaa270000000cfff3,
    0x53cc0032fc34000a,
    0x478fe97a6b0a807f,
    0xb1d37ebee6ba24d7,
    0x8ec9733bbf78ab2f,
    0x9d645513d83de7e,
]);

impl G1Affine {
    /// Bytes size of the raw representation
    pub const RAW_SIZE: usize = 97;

    /// Returns the identity of the group: the point at infinity.
    pub fn identity() -> G1Affine {
        G1Affine {
            x: Fp::zero(),
            y: Fp::one(),
            infinity: Choice::from(1u8),
        }
    }

    /// Returns a fixed generator of the group. See [`notes::design`](notes/design/index.html#fixed-generators)
    /// for how this generator is chosen.
    pub fn generator() -> G1Affine {
        G1Affine {
            x: Fp::from_raw_unchecked([
                0x5cb38790fd530c16,
                0x7817fc679976fff5,
                0x154f95c7143ba1c1,
                0xf0ae6acdf3d0e747,
                0xedce6ecc21dbf440,
                0x120177419e0bfb75,
            ]),
            y: Fp::from_raw_unchecked([
                0xbaac93d50ce72271,
                0x8c22631a7918fd8e,
                0xdd595f13570725ce,
                0x51ac582950405194,
                0xe1c8c3fad0059c0,
                0xbbc3efc5008a26a,
            ]),
            infinity: Choice::from(0u8),
        }
    }

    /// Raw bytes representation
    ///
    /// The intended usage of this function is for trusted sets of data where performance is
    /// critical.
    ///
    /// For secure serialization, check `to_bytes`
    pub fn to_raw_bytes(&self) -> [u8; Self::RAW_SIZE] {
        let mut bytes = [0u8; Self::RAW_SIZE];
        let chunks = bytes.chunks_mut(8);

        self.x
            .internal_repr()
            .iter()
            .chain(self.y.internal_repr().iter())
            .zip(chunks)
            .for_each(|(n, c)| c.copy_from_slice(&n.to_le_bytes()));

        bytes[Self::RAW_SIZE - 1] = self.infinity.unwrap_u8();

        bytes
    }

    /// Create a `G1Affine` from a set of bytes created by `G1Affine::to_raw_bytes`.
    ///
    /// No check is performed and no constant time is granted. The expected usage of this function
    /// is for trusted bytes where performance is critical.
    ///
    /// For secure serialization, check `from_bytes`
    ///
    /// After generating the point, you can check `is_on_curve` and `is_torsion_free` to grant its
    /// security
    pub unsafe fn from_slice_unchecked(bytes: &[u8]) -> Self {
        let mut x = [0u64; 6];
        let mut y = [0u64; 6];
        let mut z = [0u8; 8];

        bytes
            .chunks_exact(8)
            .zip(x.iter_mut().chain(y.iter_mut()))
            .for_each(|(c, n)| {
                z.copy_from_slice(c);
                *n = u64::from_le_bytes(z);
            });

        let x = Fp::from_raw_unchecked(x);
        let y = Fp::from_raw_unchecked(y);

        let infinity = if bytes.len() >= Self::RAW_SIZE {
            bytes[Self::RAW_SIZE - 1].into()
        } else {
            0u8.into()
        };

        Self { x, y, infinity }
    }

    /// Returns true if this element is the identity (the point at infinity).
    #[inline]
    pub fn is_identity(&self) -> Choice {
        self.infinity
    }

    /// Returns true if this point is free of an $h$-torsion component, and so it
    /// exists within the $q$-order subgroup $\mathbb{G}_1$. This should always return true
    /// unless an "unchecked" API was used.
    pub fn is_torsion_free(&self) -> Choice {
        // Algorithm from Section 6 of https://eprint.iacr.org/2021/1130
        // Updated proof of correctness in https://eprint.iacr.org/2022/352
        //
        // Check that endomorphism_p(P) == -[x^2] P

        let minus_x_squared_times_p = G1Projective::from(self).mul_by_x().mul_by_x().neg();
        let endomorphism_p = endomorphism(self);
        minus_x_squared_times_p.ct_eq(&G1Projective::from(endomorphism_p))
    }

    /// Returns true if this point is on the curve. This should always return
    /// true unless an "unchecked" API was used.
    pub fn is_on_curve(&self) -> Choice {
        // y^2 - x^3 ?= 4
        (self.y.square() - (self.x.square() * self.x)).ct_eq(&B) | self.infinity
    }
}

/// A nontrivial third root of unity in Fp
pub const BETA: Fp = Fp::from_raw_unchecked([
    0x30f1_361b_798a_64e8,
    0xf3b8_ddab_7ece_5a2a,
    0x16a8_ca3a_c615_77f7,
    0xc26a_2ff8_74fd_029b,
    0x3636_b766_6070_1c6e,
    0x051b_a4ab_241b_6160,
]);

fn endomorphism(p: &G1Affine) -> G1Affine {
    // Endomorphism of the points on the curve.
    // endomorphism_p(x,y) = (BETA * x, y)
    // where BETA is a non-trivial cubic root of unity in Fq.
    let mut res = p.clone();
    res.x *= BETA;
    res
}

/// This is an element of $\mathbb{G}_1$ represented in the projective coordinate space.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "canon", derive(Canon))]
#[cfg_attr(feature = "rkyv-impl", derive(Archive, RkyvSerialize, RkyvDeserialize))]
#[cfg_attr(feature = "rkyv-impl", archive_attr(derive(CheckBytes)))]
pub struct G1Projective {
    x: Fp,
    y: Fp,
    z: Fp,
}

impl<'a> From<&'a G1Affine> for G1Projective {
    fn from(p: &'a G1Affine) -> G1Projective {
        G1Projective {
            x: p.x,
            y: p.y,
            z: Fp::conditional_select(&Fp::one(), &Fp::zero(), p.infinity),
        }
    }
}

impl From<G1Affine> for G1Projective {
    fn from(p: G1Affine) -> G1Projective {
        G1Projective::from(&p)
    }
}

impl ConstantTimeEq for G1Projective {
    fn ct_eq(&self, other: &Self) -> Choice {
        // Is (xz^2, yz^3, z) equal to (x'z'^2, yz'^3, z') when converted to affine?

        let z = other.z.square();
        let x1 = self.x * z;
        let z = z * other.z;
        let y1 = self.y * z;
        let z = self.z.square();
        let x2 = other.x * z;
        let z = z * self.z;
        let y2 = other.y * z;

        let self_is_zero = self.z.is_zero();
        let other_is_zero = other.z.is_zero();

        (self_is_zero & other_is_zero) // Both point at infinity
            | ((!self_is_zero) & (!other_is_zero) & x1.ct_eq(&x2) & y1.ct_eq(&y2))
        // Neither point at infinity, coordinates are the same
    }
}

impl ConditionallySelectable for G1Projective {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        G1Projective {
            x: Fp::conditional_select(&a.x, &b.x, choice),
            y: Fp::conditional_select(&a.y, &b.y, choice),
            z: Fp::conditional_select(&a.z, &b.z, choice),
        }
    }
}

impl Eq for G1Projective {}
impl PartialEq for G1Projective {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        bool::from(self.ct_eq(other))
    }
}

impl<'a> Neg for &'a G1Projective {
    type Output = G1Projective;

    #[inline]
    fn neg(self) -> G1Projective {
        G1Projective {
            x: self.x,
            y: -self.y,
            z: self.z,
        }
    }
}

impl Neg for G1Projective {
    type Output = G1Projective;

    #[inline]
    fn neg(self) -> G1Projective {
        -&self
    }
}

impl<'a, 'b> Add<&'b G1Projective> for &'a G1Projective {
    type Output = G1Projective;

    #[inline]
    fn add(self, rhs: &'b G1Projective) -> G1Projective {
        self.add(rhs)
    }
}

impl<'a, 'b> Sub<&'b G1Projective> for &'a G1Projective {
    type Output = G1Projective;

    #[inline]
    fn sub(self, rhs: &'b G1Projective) -> G1Projective {
        self + (-rhs)
    }
}

impl<'a, 'b> Mul<&'b BlsScalar> for &'a G1Projective {
    type Output = G1Projective;

    fn mul(self, other: &'b BlsScalar) -> Self::Output {
        self.multiply(&other.to_bytes())
    }
}

impl<'a, 'b> Mul<&'b BlsScalar> for &'a G1Affine {
    type Output = G1Projective;

    fn mul(self, other: &'b BlsScalar) -> Self::Output {
        G1Projective::from(self).multiply(&other.to_bytes())
    }
}

impl_binops_additive!(G1Projective, G1Projective);
impl_binops_multiplicative!(G1Projective, BlsScalar);
impl_binops_multiplicative_mixed!(G1Affine, BlsScalar, G1Projective);

impl G1Projective {
    /// Returns the identity of the group: the point at infinity.
    pub fn identity() -> G1Projective {
        G1Projective {
            x: Fp::zero(),
            y: Fp::one(),
            z: Fp::zero(),
        }
    }

    /// Returns a fixed generator of the group. See [`notes::design`](notes/design/index.html#fixed-generators)
    /// for how this generator is chosen.
    pub fn generator() -> G1Projective {
        G1Projective {
            x: Fp::from_raw_unchecked([
                0x5cb38790fd530c16,
                0x7817fc679976fff5,
                0x154f95c7143ba1c1,
                0xf0ae6acdf3d0e747,
                0xedce6ecc21dbf440,
                0x120177419e0bfb75,
            ]),
            y: Fp::from_raw_unchecked([
                0xbaac93d50ce72271,
                0x8c22631a7918fd8e,
                0xdd595f13570725ce,
                0x51ac582950405194,
                0xe1c8c3fad0059c0,
                0xbbc3efc5008a26a,
            ]),
            z: Fp::one(),
        }
    }

    /// Computes the doubling of this point.
    pub fn double(&self) -> G1Projective {
        // http://www.hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#doubling-dbl-2009-l
        //
        // There are no points of order 2.

        let a = self.x.square();
        let b = self.y.square();
        let c = b.square();
        let d = self.x + b;
        let d = d.square();
        let d = d - a - c;
        let d = d + d;
        let e = a + a + a;
        let f = e.square();
        let z3 = self.z * self.y;
        let z3 = z3 + z3;
        let x3 = f - (d + d);
        let c = c + c;
        let c = c + c;
        let c = c + c;
        let y3 = e * (d - x3) - c;

        let tmp = G1Projective {
            x: x3,
            y: y3,
            z: z3,
        };

        G1Projective::conditional_select(&tmp, &G1Projective::identity(), self.is_identity())
    }

    /// Adds this point to another point.
    pub fn add(&self, rhs: &G1Projective) -> G1Projective {
        // This Jacobian point addition technique is based on the implementation in libsecp256k1,
        // which assumes that rhs has z=1. Let's address the case of zero z-coordinates generally.

        // If self is the identity, return rhs. Otherwise, return self. The other cases will be
        // predicated on neither self nor rhs being the identity.
        let f1 = self.is_identity();
        let res = G1Projective::conditional_select(self, rhs, f1);
        let f2 = rhs.is_identity();

        // If neither are the identity but x1 = x2 and y1 != y2, then return the identity
        let z = rhs.z.square();
        let u1 = self.x * z;
        let z = z * rhs.z;
        let s1 = self.y * z;
        let z = self.z.square();
        let u2 = rhs.x * z;
        let z = z * self.z;
        let s2 = rhs.y * z;
        let f3 = u1.ct_eq(&u2) & (!s1.ct_eq(&s2));
        let res =
            G1Projective::conditional_select(&res, &G1Projective::identity(), (!f1) & (!f2) & f3);

        let t = u1 + u2;
        let m = s1 + s2;
        let rr = t.square();
        let m_alt = -u2;
        let tt = u1 * m_alt;
        let rr = rr + tt;

        // Correct for x1 != x2 but y1 = -y2, which can occur because p - 1 is divisible by 3.
        // libsecp256k1 does this by substituting in an alternative (defined) expression for lambda.
        let degenerate = m.is_zero() & rr.is_zero();
        let rr_alt = s1 + s1;
        let m_alt = m_alt + u1;
        let rr_alt = Fp::conditional_select(&rr_alt, &rr, !degenerate);
        let m_alt = Fp::conditional_select(&m_alt, &m, !degenerate);

        let n = m_alt.square();
        let q = n * t;

        let n = n.square();
        let n = Fp::conditional_select(&n, &m, degenerate);
        let t = rr_alt.square();
        let z3 = m_alt * self.z * rhs.z; // We allow rhs.z != 1, so we must account for this.
        let z3 = z3 + z3;
        let q = -q;
        let t = t + q;
        let x3 = t;
        let t = t + t;
        let t = t + q;
        let t = t * rr_alt;
        let t = t + n;
        let y3 = -t;
        let x3 = x3 + x3;
        let x3 = x3 + x3;
        let y3 = y3 + y3;
        let y3 = y3 + y3;

        let tmp = G1Projective {
            x: x3,
            y: y3,
            z: z3,
        };

        G1Projective::conditional_select(&res, &tmp, (!f1) & (!f2) & (!f3))
    }

    /// Adds this point to another point in the affine model.
    pub fn add_mixed(&self, rhs: &G1Affine) -> G1Projective {
        // This Jacobian point addition technique is based on the implementation in libsecp256k1,
        // which assumes that rhs has z=1. Let's address the case of zero z-coordinates generally.

        // If self is the identity, return rhs. Otherwise, return self. The other cases will be
        // predicated on neither self nor rhs being the identity.
        let f1 = self.is_identity();
        let res = G1Projective::conditional_select(self, &G1Projective::from(rhs), f1);
        let f2 = rhs.is_identity();

        // If neither are the identity but x1 = x2 and y1 != y2, then return the identity
        let u1 = self.x;
        let s1 = self.y;
        let z = self.z.square();
        let u2 = rhs.x * z;
        let z = z * self.z;
        let s2 = rhs.y * z;
        let f3 = u1.ct_eq(&u2) & (!s1.ct_eq(&s2));
        let res =
            G1Projective::conditional_select(&res, &G1Projective::identity(), (!f1) & (!f2) & f3);

        let t = u1 + u2;
        let m = s1 + s2;
        let rr = t.square();
        let m_alt = -u2;
        let tt = u1 * m_alt;
        let rr = rr + tt;

        // Correct for x1 != x2 but y1 = -y2, which can occur because p - 1 is divisible by 3.
        // libsecp256k1 does this by substituting in an alternative (defined) expression for lambda.
        let degenerate = m.is_zero() & rr.is_zero();
        let rr_alt = s1 + s1;
        let m_alt = m_alt + u1;
        let rr_alt = Fp::conditional_select(&rr_alt, &rr, !degenerate);
        let m_alt = Fp::conditional_select(&m_alt, &m, !degenerate);

        let n = m_alt.square();
        let q = n * t;

        let n = n.square();
        let n = Fp::conditional_select(&n, &m, degenerate);
        let t = rr_alt.square();
        let z3 = m_alt * self.z;
        let z3 = z3 + z3;
        let q = -q;
        let t = t + q;
        let x3 = t;
        let t = t + t;
        let t = t + q;
        let t = t * rr_alt;
        let t = t + n;
        let y3 = -t;
        let x3 = x3 + x3;
        let x3 = x3 + x3;
        let y3 = y3 + y3;
        let y3 = y3 + y3;

        let tmp = G1Projective {
            x: x3,
            y: y3,
            z: z3,
        };

        G1Projective::conditional_select(&res, &tmp, (!f1) & (!f2) & (!f3))
    }

    fn multiply(&self, by: &[u8; 32]) -> G1Projective {
        let mut acc = G1Projective::identity();

        // This is a simple double-and-add implementation of point
        // multiplication, moving from most significant to least
        // significant bit of the scalar.
        //
        // We skip the leading bit because it's always unset for Fq
        // elements.
        for bit in by
            .iter()
            .rev()
            .flat_map(|byte| (0..8).rev().map(move |i| Choice::from((byte >> i) & 1u8)))
            .skip(1)
        {
            acc = acc.double();
            acc = G1Projective::conditional_select(&acc, &(acc + self), bit);
        }

        acc
    }

    /// Multiply `self` by `crate::BLS_X`, using double and add.
    fn mul_by_x(&self) -> G1Projective {
        let mut xself = G1Projective::identity();
        // NOTE: in BLS12-381 we can just skip the first bit.
        let mut x = crate::BLS_X >> 1;
        let mut tmp = *self;
        while x != 0 {
            tmp = tmp.double();

            if x % 2 == 1 {
                xself += tmp;
            }
            x >>= 1;
        }
        // finally, flip the sign
        if crate::BLS_X_IS_NEGATIVE {
            xself = -xself;
        }
        xself
    }

    /// Multiplies by $(1 - z)$, where $z$ is the parameter of BLS12-381, which
    /// [suffices to clear](https://ia.cr/2019/403) the cofactor and map
    /// elliptic curve points to elements of $\mathbb{G}\_1$.
    pub fn clear_cofactor(&self) -> G1Projective {
        self - self.mul_by_x()
    }

    /// Converts a batch of `G1Projective` elements into `G1Affine` elements. This
    /// function will panic if `p.len() != q.len()`.
    pub fn batch_normalize(p: &[Self], q: &mut [G1Affine]) {
        assert_eq!(p.len(), q.len());

        let mut acc = Fp::one();
        for (p, q) in p.iter().zip(q.iter_mut()) {
            // We use the `x` field of `G1Affine` to store the product
            // of previous z-coordinates seen.
            q.x = acc;

            // We will end up skipping all identities in p
            acc = Fp::conditional_select(&(acc * p.z), &acc, p.is_identity());
        }

        // This is the inverse, as all z-coordinates are nonzero and the ones
        // that are not are skipped.
        acc = acc.invert().unwrap();

        for (p, q) in p.iter().rev().zip(q.iter_mut().rev()) {
            let skip = p.is_identity();

            // Compute tmp = 1/z
            let tmp = q.x * acc;

            // Cancel out z-coordinate in denominator of `acc`
            acc = Fp::conditional_select(&(acc * p.z), &acc, skip);

            // Set the coordinates to the correct value
            let tmp2 = tmp.square();
            let tmp3 = tmp2 * tmp;

            q.x = p.x * tmp2;
            q.y = p.y * tmp3;
            q.infinity = Choice::from(0u8);

            *q = G1Affine::conditional_select(&q, &G1Affine::identity(), skip);
        }
    }

    /// Returns true if this element is the identity (the point at infinity).
    #[inline]
    pub fn is_identity(&self) -> Choice {
        self.z.is_zero()
    }

    /// Returns true if this point is on the curve. This should always return
    /// true unless an "unchecked" API was used.
    pub fn is_on_curve(&self) -> Choice {
        // Y^2 - X^3 = 4(Z^6)

        (self.y.square() - (self.x.square() * self.x))
            .ct_eq(&((self.z.square() * self.z).square() * B))
            | self.z.is_zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beta() {
        assert_eq!(
            BETA,
            Fp::from_bytes(&[
                0x00u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5f, 0x19, 0x67, 0x2f, 0xdf,
                0x76, 0xce, 0x51, 0xba, 0x69, 0xc6, 0x07, 0x6a, 0x0f, 0x77, 0xea, 0xdd, 0xb3, 0xa9,
                0x3b, 0xe6, 0xf8, 0x96, 0x88, 0xde, 0x17, 0xd8, 0x13, 0x62, 0x0a, 0x00, 0x02, 0x2e,
                0x01, 0xff, 0xff, 0xff, 0xfe, 0xff, 0xfe
            ])
            .unwrap()
        );
        assert_ne!(BETA, Fp::one());
        assert_ne!(BETA * BETA, Fp::one());
        assert_eq!(BETA * BETA * BETA, Fp::one());
    }

    #[test]
    fn test_is_on_curve() {
        assert!(bool::from(G1Affine::identity().is_on_curve()));
        assert!(bool::from(G1Affine::generator().is_on_curve()));
        assert!(bool::from(G1Projective::identity().is_on_curve()));
        assert!(bool::from(G1Projective::generator().is_on_curve()));

        let z = Fp::from_raw_unchecked([
            0xba7afa1f9a6fe250,
            0xfa0f5b595eafe731,
            0x3bdc477694c306e7,
            0x2149be4b3949fa24,
            0x64aa6e0649b2078c,
            0x12b108ac33643c3e,
        ]);

        let gen = G1Affine::generator();
        let mut test = G1Projective {
            x: gen.x * (z.square()),
            y: gen.y * (z.square() * z),
            z,
        };

        assert!(bool::from(test.is_on_curve()));

        test.x = z;
        assert!(!bool::from(test.is_on_curve()));
    }

    #[test]
    fn test_affine_point_equality() {
        let a = G1Affine::generator();
        let b = G1Affine::identity();

        assert!(a == a);
        assert!(b == b);
        assert!(a != b);
        assert!(b != a);
    }

    #[test]
    fn test_projective_point_equality() {
        let a = G1Projective::generator();
        let b = G1Projective::identity();

        assert!(a == a);
        assert!(b == b);
        assert!(a != b);
        assert!(b != a);

        let z = Fp::from_raw_unchecked([
            0xba7afa1f9a6fe250,
            0xfa0f5b595eafe731,
            0x3bdc477694c306e7,
            0x2149be4b3949fa24,
            0x64aa6e0649b2078c,
            0x12b108ac33643c3e,
        ]);

        let mut c = G1Projective {
            x: a.x * (z.square()),
            y: a.y * (z.square() * z),
            z,
        };
        assert!(bool::from(c.is_on_curve()));

        assert!(a == c);
        assert!(b != c);
        assert!(c == a);
        assert!(c != b);

        c.y = -c.y;
        assert!(bool::from(c.is_on_curve()));

        assert!(a != c);
        assert!(b != c);
        assert!(c != a);
        assert!(c != b);

        c.y = -c.y;
        c.x = z;
        assert!(!bool::from(c.is_on_curve()));
        assert!(a != b);
        assert!(a != c);
        assert!(b != c);
    }

    #[test]
    fn test_conditionally_select_affine() {
        let a = G1Affine::generator();
        let b = G1Affine::identity();

        assert_eq!(G1Affine::conditional_select(&a, &b, Choice::from(0u8)), a);
        assert_eq!(G1Affine::conditional_select(&a, &b, Choice::from(1u8)), b);
    }

    #[test]
    fn test_conditionally_select_projective() {
        let a = G1Projective::generator();
        let b = G1Projective::identity();

        assert_eq!(
            G1Projective::conditional_select(&a, &b, Choice::from(0u8)),
            a
        );
        assert_eq!(
            G1Projective::conditional_select(&a, &b, Choice::from(1u8)),
            b
        );
    }

    #[test]
    fn test_projective_to_affine() {
        let a = G1Projective::generator();
        let b = G1Projective::identity();

        assert!(bool::from(G1Affine::from(a).is_on_curve()));
        assert!(!bool::from(G1Affine::from(a).is_identity()));
        assert!(bool::from(G1Affine::from(b).is_on_curve()));
        assert!(bool::from(G1Affine::from(b).is_identity()));

        let z = Fp::from_raw_unchecked([
            0xba7afa1f9a6fe250,
            0xfa0f5b595eafe731,
            0x3bdc477694c306e7,
            0x2149be4b3949fa24,
            0x64aa6e0649b2078c,
            0x12b108ac33643c3e,
        ]);

        let c = G1Projective {
            x: a.x * (z.square()),
            y: a.y * (z.square() * z),
            z,
        };

        assert_eq!(G1Affine::from(c), G1Affine::generator());
    }

    #[test]
    fn test_affine_to_projective() {
        let a = G1Affine::generator();
        let b = G1Affine::identity();

        assert!(bool::from(G1Projective::from(a).is_on_curve()));
        assert!(!bool::from(G1Projective::from(a).is_identity()));
        assert!(bool::from(G1Projective::from(b).is_on_curve()));
        assert!(bool::from(G1Projective::from(b).is_identity()));
    }

    #[test]
    fn test_doubling() {
        {
            let tmp = G1Projective::identity().double();
            assert!(bool::from(tmp.is_identity()));
            assert!(bool::from(tmp.is_on_curve()));
        }
        {
            let tmp = G1Projective::generator().double();
            assert!(!bool::from(tmp.is_identity()));
            assert!(bool::from(tmp.is_on_curve()));

            assert_eq!(
                G1Affine::from(tmp),
                G1Affine {
                    x: Fp::from_raw_unchecked([
                        0x53e978ce58a9ba3c,
                        0x3ea0583c4f3d65f9,
                        0x4d20bb47f0012960,
                        0xa54c664ae5b2b5d9,
                        0x26b552a39d7eb21f,
                        0x8895d26e68785
                    ]),
                    y: Fp::from_raw_unchecked([
                        0x70110b3298293940,
                        0xda33c5393f1f6afc,
                        0xb86edfd16a5aa785,
                        0xaec6d1c9e7b1c895,
                        0x25cfc2b522d11720,
                        0x6361c83f8d09b15
                    ]),
                    infinity: Choice::from(0u8)
                }
            );
        }
    }

    #[test]
    fn test_projective_addition() {
        {
            let a = G1Projective::identity();
            let b = G1Projective::identity();
            let c = a + b;
            assert!(bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
        }
        {
            let a = G1Projective::identity();
            let mut b = G1Projective::generator();
            {
                let z = Fp::from_raw_unchecked([
                    0xba7afa1f9a6fe250,
                    0xfa0f5b595eafe731,
                    0x3bdc477694c306e7,
                    0x2149be4b3949fa24,
                    0x64aa6e0649b2078c,
                    0x12b108ac33643c3e,
                ]);

                b = G1Projective {
                    x: b.x * (z.square()),
                    y: b.y * (z.square() * z),
                    z,
                };
            }
            let c = a + b;
            assert!(!bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
            assert!(c == G1Projective::generator());
        }
        {
            let a = G1Projective::identity();
            let mut b = G1Projective::generator();
            {
                let z = Fp::from_raw_unchecked([
                    0xba7afa1f9a6fe250,
                    0xfa0f5b595eafe731,
                    0x3bdc477694c306e7,
                    0x2149be4b3949fa24,
                    0x64aa6e0649b2078c,
                    0x12b108ac33643c3e,
                ]);

                b = G1Projective {
                    x: b.x * (z.square()),
                    y: b.y * (z.square() * z),
                    z,
                };
            }
            let c = b + a;
            assert!(!bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
            assert!(c == G1Projective::generator());
        }
        {
            let a = G1Projective::generator().double().double(); // 4P
            let b = G1Projective::generator().double(); // 2P
            let c = a + b;

            let mut d = G1Projective::generator();
            for _ in 0..5 {
                d = d + G1Projective::generator();
            }
            assert!(!bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
            assert!(!bool::from(d.is_identity()));
            assert!(bool::from(d.is_on_curve()));
            assert_eq!(c, d);
        }

        // Degenerate case
        {
            let beta = Fp::from_raw_unchecked([
                0xcd03c9e48671f071,
                0x5dab22461fcda5d2,
                0x587042afd3851b95,
                0x8eb60ebe01bacb9e,
                0x3f97d6e83d050d2,
                0x18f0206554638741,
            ]);
            let beta = beta.square();
            let a = G1Projective::generator().double().double();
            let b = G1Projective {
                x: a.x * beta,
                y: -a.y,
                z: a.z,
            };
            assert!(bool::from(a.is_on_curve()));
            assert!(bool::from(b.is_on_curve()));

            let c = a + b;
            assert_eq!(
                G1Affine::from(c),
                G1Affine::from(G1Projective {
                    x: Fp::from_raw_unchecked([
                        0x29e1e987ef68f2d0,
                        0xc5f3ec531db03233,
                        0xacd6c4b6ca19730f,
                        0x18ad9e827bc2bab7,
                        0x46e3b2c5785cc7a9,
                        0x7e571d42d22ddd6
                    ]),
                    y: Fp::from_raw_unchecked([
                        0x94d117a7e5a539e7,
                        0x8e17ef673d4b5d22,
                        0x9d746aaf508a33ea,
                        0x8c6d883d2516c9a2,
                        0xbc3b8d5fb0447f7,
                        0x7bfa4c7210f4f44
                    ]),
                    z: Fp::one()
                })
            );
            assert!(!bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
        }
    }

    #[test]
    fn test_mixed_addition() {
        {
            let a = G1Affine::identity();
            let b = G1Projective::identity();
            let c = a + b;
            assert!(bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
        }
        {
            let a = G1Affine::identity();
            let mut b = G1Projective::generator();
            {
                let z = Fp::from_raw_unchecked([
                    0xba7afa1f9a6fe250,
                    0xfa0f5b595eafe731,
                    0x3bdc477694c306e7,
                    0x2149be4b3949fa24,
                    0x64aa6e0649b2078c,
                    0x12b108ac33643c3e,
                ]);

                b = G1Projective {
                    x: b.x * (z.square()),
                    y: b.y * (z.square() * z),
                    z,
                };
            }
            let c = a + b;
            assert!(!bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
            assert!(c == G1Projective::generator());
        }
        {
            let a = G1Affine::identity();
            let mut b = G1Projective::generator();
            {
                let z = Fp::from_raw_unchecked([
                    0xba7afa1f9a6fe250,
                    0xfa0f5b595eafe731,
                    0x3bdc477694c306e7,
                    0x2149be4b3949fa24,
                    0x64aa6e0649b2078c,
                    0x12b108ac33643c3e,
                ]);

                b = G1Projective {
                    x: b.x * (z.square()),
                    y: b.y * (z.square() * z),
                    z,
                };
            }
            let c = b + a;
            assert!(!bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
            assert!(c == G1Projective::generator());
        }
        {
            let a = G1Projective::generator().double().double(); // 4P
            let b = G1Projective::generator().double(); // 2P
            let c = a + b;

            let mut d = G1Projective::generator();
            for _ in 0..5 {
                d = d + G1Affine::generator();
            }
            assert!(!bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
            assert!(!bool::from(d.is_identity()));
            assert!(bool::from(d.is_on_curve()));
            assert_eq!(c, d);
        }

        // Degenerate case
        {
            let beta = Fp::from_raw_unchecked([
                0xcd03c9e48671f071,
                0x5dab22461fcda5d2,
                0x587042afd3851b95,
                0x8eb60ebe01bacb9e,
                0x3f97d6e83d050d2,
                0x18f0206554638741,
            ]);
            let beta = beta.square();
            let a = G1Projective::generator().double().double();
            let b = G1Projective {
                x: a.x * beta,
                y: -a.y,
                z: a.z,
            };
            let a = G1Affine::from(a);
            assert!(bool::from(a.is_on_curve()));
            assert!(bool::from(b.is_on_curve()));

            let c = a + b;
            assert_eq!(
                G1Affine::from(c),
                G1Affine::from(G1Projective {
                    x: Fp::from_raw_unchecked([
                        0x29e1e987ef68f2d0,
                        0xc5f3ec531db03233,
                        0xacd6c4b6ca19730f,
                        0x18ad9e827bc2bab7,
                        0x46e3b2c5785cc7a9,
                        0x7e571d42d22ddd6
                    ]),
                    y: Fp::from_raw_unchecked([
                        0x94d117a7e5a539e7,
                        0x8e17ef673d4b5d22,
                        0x9d746aaf508a33ea,
                        0x8c6d883d2516c9a2,
                        0xbc3b8d5fb0447f7,
                        0x7bfa4c7210f4f44
                    ]),
                    z: Fp::one()
                })
            );
            assert!(!bool::from(c.is_identity()));
            assert!(bool::from(c.is_on_curve()));
        }
    }

    #[test]
    fn test_projective_negation_and_subtraction() {
        let a = G1Projective::generator().double();
        assert_eq!(a + (-a), G1Projective::identity());
        assert_eq!(a + (-a), a - a);
    }

    #[test]
    fn test_affine_negation_and_subtraction() {
        let a = G1Affine::generator();
        assert_eq!(G1Projective::from(a) + (-a), G1Projective::identity());
        assert_eq!(G1Projective::from(a) + (-a), G1Projective::from(a) - a);
    }

    #[test]
    fn test_projective_scalar_multiplication() {
        let g = G1Projective::generator();
        let a = BlsScalar::from_raw([
            0x2b568297a56da71c,
            0xd8c39ecb0ef375d1,
            0x435c38da67bfbf96,
            0x8088a05026b659b2,
        ]);
        let b = BlsScalar::from_raw([
            0x785fdd9b26ef8b85,
            0xc997f25837695c18,
            0x4c8dbc39e7b756c1,
            0x70d9b6cc6d87df20,
        ]);
        let c = a * b;

        assert_eq!((g * a) * b, g * c);
    }

    #[test]
    fn test_affine_scalar_multiplication() {
        let g = G1Affine::generator();
        let a = BlsScalar::from_raw([
            0x2b568297a56da71c,
            0xd8c39ecb0ef375d1,
            0x435c38da67bfbf96,
            0x8088a05026b659b2,
        ]);
        let b = BlsScalar::from_raw([
            0x785fdd9b26ef8b85,
            0xc997f25837695c18,
            0x4c8dbc39e7b756c1,
            0x70d9b6cc6d87df20,
        ]);
        let c = a * b;

        assert_eq!(G1Affine::from(g * a) * b, g * c);
    }

    #[test]
    fn test_is_torsion_free() {
        let a = G1Affine {
            x: Fp::from_raw_unchecked([
                0xabaf895b97e43c8,
                0xba4c6432eb9b61b0,
                0x12506f52adfe307f,
                0x75028c3439336b72,
                0x84744f05b8e9bd71,
                0x113d554fb09554f7,
            ]),
            y: Fp::from_raw_unchecked([
                0x73e90e88f5cf01c0,
                0x37007b65dd3197e2,
                0x5cf9a1992f0d7c78,
                0x4f83c10b9eb3330d,
                0xf6a63f6f07f60961,
                0xc53b5b97e634df3,
            ]),
            infinity: Choice::from(0u8),
        };
        assert!(!bool::from(a.is_torsion_free()));

        assert!(bool::from(G1Affine::identity().is_torsion_free()));
        assert!(bool::from(G1Affine::generator().is_torsion_free()));
    }

    #[test]
    fn test_mul_by_x() {
        // multiplying by `x` a point in G1 is the same as multiplying by
        // the equivalent scalar.
        let generator = G1Projective::generator();
        let x = if crate::BLS_X_IS_NEGATIVE {
            -BlsScalar::from(crate::BLS_X)
        } else {
            BlsScalar::from(crate::BLS_X)
        };
        assert_eq!(generator.mul_by_x(), generator * x);

        let point = G1Projective::generator() * BlsScalar::from(42);
        assert_eq!(point.mul_by_x(), point * x);
    }

    #[test]
    fn test_clear_cofactor() {
        // the generator (and the identity) are always on the curve,
        // even after clearing the cofactor
        let generator = G1Projective::generator();
        assert!(bool::from(generator.clear_cofactor().is_on_curve()));
        let id = G1Projective::identity();
        assert!(bool::from(id.clear_cofactor().is_on_curve()));

        let point = G1Projective {
            x: Fp::from_raw_unchecked([
                0x48af5ff540c817f0,
                0xd73893acaf379d5a,
                0xe6c43584e18e023c,
                0x1eda39c30f188b3e,
                0xf618c6d3ccc0f8d8,
                0x0073542cd671e16c,
            ]),
            y: Fp::from_raw_unchecked([
                0x57bf8be79461d0ba,
                0xfc61459cee3547c3,
                0x0d23567df1ef147b,
                0x0ee187bcce1d9b64,
                0xb0c8cfbe9dc8fdc1,
                0x1328661767ef368b,
            ]),
            z: Fp::from_raw_unchecked([
                0x3d2d1c670671394e,
                0x0ee3a800a2f7c1ca,
                0x270f4f21da2e5050,
                0xe02840a53f1be768,
                0x55debeb597512690,
                0x08bd25353dc8f791,
            ]),
        };

        assert!(bool::from(point.is_on_curve()));
        assert!(!bool::from(G1Affine::from(point).is_torsion_free()));
        let cleared_point = point.clear_cofactor();
        assert!(bool::from(cleared_point.is_on_curve()));
        assert!(bool::from(G1Affine::from(cleared_point).is_torsion_free()));

        // in BLS12-381 the cofactor in G1 can be
        // cleared multiplying by (1-x)
        let h_eff = BlsScalar::from(1) + BlsScalar::from(crate::BLS_X);
        assert_eq!(point.clear_cofactor(), point * h_eff);
    }

    #[test]
    fn test_batch_normalize() {
        let a = G1Projective::generator().double();
        let b = a.double();
        let c = b.double();

        for a_identity in (0..1).map(|n| n == 1) {
            for b_identity in (0..1).map(|n| n == 1) {
                for c_identity in (0..1).map(|n| n == 1) {
                    let mut v = [a, b, c];
                    if a_identity {
                        v[0] = G1Projective::identity()
                    }
                    if b_identity {
                        v[1] = G1Projective::identity()
                    }
                    if c_identity {
                        v[2] = G1Projective::identity()
                    }

                    let mut t = [
                        G1Affine::identity(),
                        G1Affine::identity(),
                        G1Affine::identity(),
                    ];
                    let expected = [
                        G1Affine::from(v[0]),
                        G1Affine::from(v[1]),
                        G1Affine::from(v[2]),
                    ];

                    G1Projective::batch_normalize(&v[..], &mut t[..]);

                    assert_eq!(&t[..], &expected[..]);
                }
            }
        }
    }

    #[test]
    #[cfg(feature = "serde_req")]
    fn g1_affine_serde_roundtrip() {
        use bincode;

        let gen = G1Affine::generator();
        let ser = bincode::serialize(&gen).unwrap();
        let deser: G1Affine = bincode::deserialize(&ser).unwrap();

        assert_eq!(gen, deser);
    }

    #[test]
    fn g1_affine_bytes_unchecked() {
        let gen = G1Affine::generator();
        let ident = G1Affine::identity();

        let gen_p = gen.to_raw_bytes();
        let gen_p = unsafe { G1Affine::from_slice_unchecked(&gen_p) };

        let ident_p = ident.to_raw_bytes();
        let ident_p = unsafe { G1Affine::from_slice_unchecked(&ident_p) };

        assert_eq!(gen, gen_p);
        assert_eq!(ident, ident_p);
    }

    #[test]
    fn g1_affine_bytes_unchecked_field() {
        let x = Fp::from_raw_unchecked([
            0x9af1f35780fffb82,
            0x557416ceeea5a52f,
            0x1e4403e4911a2d97,
            0xb85bfb438316bf2,
            0xa3b716c69a9e5a7b,
            0x1fe9b8ad976dd39,
        ]);

        let y = Fp::from_raw_unchecked([
            0xb4f1cc806acfb4e2,
            0x38c28cba4cf600ed,
            0x3af1c2f54a01a366,
            0x96a75ac708a9eb72,
            0x4253bd59228e50d,
            0x120114fae4294c21,
        ]);

        let infinity = 0u8.into();
        let g = G1Affine { x, y, infinity };

        let g_p = g.to_raw_bytes();
        let g_p = unsafe { G1Affine::from_slice_unchecked(&g_p) };

        assert_eq!(g, g_p);
    }

    #[test]
    fn g1_affine_hex() {
        use dusk_bytes::ParseHexStr;

        let gen = G1Affine::generator();
        let ident = G1Affine::identity();

        let gen_p = format!("{:x}", gen);
        let gen_p = G1Affine::from_hex_str(gen_p.as_str()).unwrap();

        let ident_p = format!("{:x}", ident);
        let ident_p = G1Affine::from_hex_str(ident_p.as_str()).unwrap();

        assert_eq!(gen, gen_p);
        assert_eq!(ident, ident_p);
    }
}
