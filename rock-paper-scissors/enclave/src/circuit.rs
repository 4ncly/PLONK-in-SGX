use ff::PrimeField;
use bellman::{Circuit, ConstraintSystem, SynthesisError,Variable};
use std::vec::Vec;
use std::fmt::format;
use bellman::gadgets::Assignment;
use bellman::gadgets::num::*;
use bellman::gadgets::uint32::*;
use bellman::gadgets::boolean::*;
use bellman::LinearCombination;
use bellman::gadgets::{sha256::sha256,multipack};


pub fn handle(a:u64, b:u64)->Vec<bool>
{
    let mut out = vec![];

    match a {
        1 => {out.push(false);out.push(true)},
        2 => {out.push(true);out.push(false)},
        3 => {out.push(true);out.push(true)},
        _ => {{out.push(false);out.push(false)}}
    }
    match b {
        1 => {out.push(false);out.push(true)},
        2 => {out.push(true);out.push(false)},
        3 => {out.push(true);out.push(true)},
        _ => {{out.push(false);out.push(false)}}
    }
    out
}

pub fn rps(a:u64, b:u64)->Vec<u64>
{
    let mut out = vec![];
    if a==b
    {
        out.push(0);
        out.push(0);
    }
    if (a>1 && a==b+1)||(a==1 && b==3)
    {
        out.push(1);
        out.push(0);
    }
    else
    {
        out.push(0);
        out.push(1);
    }
    out
}



pub fn mimc<S: PrimeField>(mut xl: S, mut xr: S, constants: &[S]) -> S {
    assert_eq!(constants.len(), 200);

    for c in constants {
        let mut tmp1 = xl;
        tmp1.add_assign(c);
        let mut tmp2 = tmp1.square();
        tmp2.mul_assign(&tmp1);
        tmp2.add_assign(&xr);
        xr = xl;
        xl = tmp2;
    }

    xl
}



pub struct Compare
{
    pub a: Option<u32>,
    pub b: Option<u32>,
}

impl<S:PrimeField> Circuit<S> for Compare
{
    fn synthesize<CS: ConstraintSystem<S>>(self, cs: &mut CS) -> Result<(), SynthesisError>{
        
        let mut var_t = Vec::with_capacity(32);
        var_t.push(self.a);
        var_t.push(self.b);

        let var = var_t.into_iter().enumerate()
        .map(|(i,v)|  cs.alloc(
            || format!("var {}",i),
            || {
                Ok(S::from(v.unwrap().into()))
            },
        )).collect::<Result<Vec<_>, SynthesisError>>()?;


        let bit_vec_a = check(self.a, cs, 0, var[0],8);

        let bit_vec_b = check(self.b, cs, 1, var[1],8);

 
        let delta = self.a.map(|e| e + 256-self.b.unwrap());

        let res_pub = cs.alloc(|| "res_pub ", || Some(S::from((delta.unwrap()).into())).ok_or(SynthesisError::AssignmentMissing))?;
        

        let bit_vec_r = check(delta, cs, 2, res_pub,9).unwrap();

        let mut lc_0 = LinearCombination::<S>::zero();
        let mut coff = S::from(1);
        for i in 0..9{

            lc_0 = lc_0 + (coff,bit_vec_r[i]);
            coff = coff.double();

        }

        cs.enforce(|| "res_check", |lc| lc+ CS::one(), |_| lc_0 , |lc| lc + (S::from(256),CS::one()) + var[0]-var[1]);

        let input = cs.alloc_input(|| "input", || Some(S::from(((delta.unwrap()>>8)&1).into())).ok_or(SynthesisError::AssignmentMissing))?;


        cs.enforce(|| "res", |lc| lc + CS::one(), |lc| lc + bit_vec_r[8], |lc| lc +input);




        Ok(())
    }
}


fn check<S:PrimeField,CS:ConstraintSystem<S>>(num : Option<u32> , cs: &mut CS , id : u32 , v:Variable , len: usize)-> Result<Vec<Variable>, SynthesisError>{

    let values = match num{
        Some(v) => {
            let mut tmp = Vec::with_capacity(32);
            for i in 0..len{
                tmp.push(Some((v>>i)&1==1));
            }
            tmp
        }
        None => vec![None;len],
    };
    
    let bits = values.into_iter().enumerate()
    .map(|(i, b)| {
        cs.alloc(
            || format!("bit {}",i),
            || {
                if  *b.get()? {
                    Ok(S::one())
                } else {
                    Ok(S::zero())
                }
            },
        )
    })
    .collect::<Result<Vec<_>, SynthesisError>>()?;

    for i in 0..len{
        cs.enforce(|| format!("check {}",i), |lc| lc+bits[i],|lc| lc + CS::one() - bits[i],|lc| lc);
    }
    
    let mut lc_0 = LinearCombination::<S>::zero();
    let mut coff = S::from(1);
    for i in 0..len{

        lc_0 = lc_0 + (coff,bits[i]);
        coff = coff.double();

    }

    cs.enforce(|| format!("check_uint32_{}",id), |lc| lc + CS::one(), |_| lc_0, |lc| lc + v );


    Ok(bits)

}



pub struct RPS
{
    pub a: Option<bool>,
    pub b: Option<bool>,
    pub c: Option<bool>,
    pub d: Option<bool>,
}

impl<S:PrimeField> Circuit<S> for RPS
{
    fn synthesize<CS: ConstraintSystem<S>>(self, cs: &mut CS) -> Result<(), SynthesisError>{
        
        let _0 = AllocatedBit::alloc(cs.namespace(|| "a"), self.a)?;
        let _1 = AllocatedBit::alloc(cs.namespace(|| "b"), self.b)?;
        let _2 = AllocatedBit::alloc(cs.namespace(|| "c"), self.c)?;
        let _3 = AllocatedBit::alloc(cs.namespace(|| "d"), self.d)?;

        let _4 = AllocatedBit::and_not(cs.namespace(|| ""), &_1, &_0)?;
        let _5 = AllocatedBit::and_not(cs.namespace(|| ""), &_4, &_2)?;
        let _6 = AllocatedBit::and(    cs.namespace(|| ""), &_5, &_3)?;
        let _7 = AllocatedBit::and_not(cs.namespace(|| ""), &_1, &_0)?;
        let _8 = AllocatedBit::and(    cs.namespace(|| ""), &_2, &_7)?;
        let _9 = AllocatedBit::and_not(cs.namespace(|| ""), &_8, &_3)?;
        let _10 = AllocatedBit::or(cs.namespace(|| ""),&_6 , &_9)?;

        let _11 = AllocatedBit::and_not(cs.namespace(|| ""), &_0, &_1)?;
        let _12 = AllocatedBit::and(cs.namespace(|| ""), &_11, &_2)?;
        let _13 = AllocatedBit::and_not(cs.namespace(|| ""), &_12, &_3)?;
        let _14 = AllocatedBit::or(cs.namespace(|| ""),&_10,&_13)?;

        let _15 = AllocatedBit::and_not(cs.namespace(|| ""), &_0, &_1)?;
        let _16 = AllocatedBit::and(cs.namespace(|| ""),&_15,&_2)?;
        let _17 = AllocatedBit::and(cs.namespace(|| ""),&_16,&_3)?;
        let _18 = AllocatedBit::or(cs.namespace(|| ""),&_14,&_17)?;

        let _19 = AllocatedBit::and(cs.namespace(|| ""),&_0,&_1)?;
        let _20 = AllocatedBit::and_not(cs.namespace(|| ""), &_19, &_2)?;
        let _21 = AllocatedBit::and(cs.namespace(|| ""),&_20,&_3)?;
        let _22 = AllocatedBit::or(cs.namespace(|| ""),&_21,&_18)?;

        let _23 = AllocatedBit::and(cs.namespace(|| ""),&_0,&_1)?;
        let _24 = AllocatedBit::and(cs.namespace(|| ""),&_23,&_2)?;
        let _25 = AllocatedBit::and(cs.namespace(|| ""),&_24,&_3)?;
        let _26 = AllocatedBit::or(cs.namespace(|| ""),&_22,&_25)?;

        let _27 = AllocatedBit::and_not(cs.namespace(|| ""), &_1, &_0)?;
        let _28 = AllocatedBit::and_not(cs.namespace(|| ""), &_27, &_2)?;
        let _29 = AllocatedBit::and(cs.namespace(|| ""),&_28,&_3)?;
        let _30 = AllocatedBit::and_not(cs.namespace(|| ""), &_1, &_0)?;
        let _31 = AllocatedBit::and(cs.namespace(|| ""),&_30,&_2)?;
        let _32 = AllocatedBit::and(cs.namespace(|| ""),&_31,&_3)?;
        let _33 = AllocatedBit::or(cs.namespace(|| ""),&_32,&_29)?;


        let _34 = AllocatedBit::and_not(cs.namespace(|| ""), &_0, &_1)?;
        let _35 = AllocatedBit::and_not(cs.namespace(|| ""), &_34, &_2)?;
        let _36 = AllocatedBit::and(cs.namespace(|| ""),&_35,&_3)?;
        let _37 = AllocatedBit::or(cs.namespace(|| ""),&_33,&_36)?;


        let _38 = AllocatedBit::and_not(cs.namespace(|| ""), &_0, &_1)?;
        let _39 = AllocatedBit::and(cs.namespace(|| ""),&_38,&_2)?;
        let _40 = AllocatedBit::and_not(cs.namespace(|| ""),&_39,&_3)?;
        let _41 = AllocatedBit::or(cs.namespace(|| ""),&_37,&_40)?;

        let _42 = AllocatedBit::and(cs.namespace(|| ""), &_1, &_0)?;
        let _43 = AllocatedBit::and(cs.namespace(|| ""),&_42,&_2)?;
        let _44 = AllocatedBit::and_not(cs.namespace(|| ""), &_43, &_3)?;
        let _45 = AllocatedBit::or(cs.namespace(|| ""),&_41,&_44)?;


        let _46 = AllocatedBit::and(cs.namespace(|| ""), &_1, &_0)?;
        let _47 = AllocatedBit::and(cs.namespace(|| ""), &_46, &_2)?;
        let _48 = AllocatedBit::and(cs.namespace(|| ""), &_47, &_3)?;
        let _49 = AllocatedBit::or(cs.namespace(|| ""),  &_45,&_48)?;


        let out_0 = cs.alloc_input(
            || "out0",
            || {
                if *_49.get_value().get()? {
                    Ok(S::one())
                } else {
                    Ok(S::zero())
                }
            },
        )?;
        cs.enforce(|| "", |lc| lc+CS::one(), |lc| lc+out_0, |lc| lc+_49.get_variable());
        
        let out_1 = cs.alloc_input(
            || "out1",
            || {
                if *_26.get_value().get()? {
                    Ok(S::one())
                } else {
                    Ok(S::zero())
                }
            },
        )?;
        cs.enforce(|| "", |lc| lc+CS::one(), |lc| lc+out_1, |lc| lc+_26.get_variable());

        Ok(())
    }
}


