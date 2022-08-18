extern crate dusk_plonk;
extern crate rand_core;
extern crate dusk_bls12_381;

mod circuit;
use circuit::*;

use rand_core::OsRng;
use dusk_plonk::prelude::*;
use dusk_bls12_381::*;

use std::time::{Instant};

fn main() {

    let mut t_0 = 0;



    let pp = PublicParameters::setup(1 << 12, &mut OsRng).unwrap();



    for i in 0..100{

        let now = Instant::now();

        // Initialize the circuit
        let mut circuit = TestCircuit::default();
        // Compile/preproces the circuit
        let (pk, vd) = circuit.compile(&pp).unwrap();
        


        // Prover POV
        let proof = {
            let mut circuit = TestCircuit {
                a: BlsScalar::from(20u64),
                b: BlsScalar::from(5u64),
                c: BlsScalar::from(25u64),
            };
            circuit.prove(&pp, &pk, b"Test", &mut OsRng).unwrap()
        };
        
        // Verifier POV
        let public_inputs: Vec<PublicInputValue> = vec![
            BlsScalar::from(25u64).into()
        ];
        let r = TestCircuit::verify(
            &pp,
            &vd,
            &proof,
            &public_inputs,
            b"Test",
        );

        // match r {
        //     Ok(()) => println!("OK"),
        //     _  => println!("Wrong")

        // }

        t_0 += now.elapsed().as_millis();

    }
    println!("{}",t_0/100);
}
