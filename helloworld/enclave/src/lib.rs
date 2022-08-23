// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

#![crate_name = "helloworldsampleenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;



use sgx_types::*;
use std::string::String;
use std::vec::Vec;
use std::slice;
use std::io::{self, Write};

extern crate rand_core;
extern crate dusk_plonk;

mod circuit;
use circuit::*;

use rand_core::*;

pub use dusk_plonk::prelude::*;
use std::time::{Duration, Instant};
use std::untrusted::time::InstantEx;

#[no_mangle]
pub extern "C" fn say_something(some_string: *const u8, some_len: usize) -> sgx_status_t {


    let mut t_0 = 0;



    let pp = PublicParameters::setup(1 << 12, &mut OsRng).unwrap();


    for i in 0..1{

        let now = Instant::now();

        // Initialize the circuit
        let mut circuit = Yao::default();
        // Compile/preproces the circuit
        let (pk, vd) = circuit.compile(&pp).unwrap();
        
        // Prover POV
        let proof = {
            let mut circuit = Yao {

                a: BlsScalar::from(12),
                b: BlsScalar::from(10),
                c: BlsScalar::from(1)

            };
            circuit.prove(&pp, &pk, b"Test", &mut OsRng).unwrap()
        };

        // Verifier POV
        let public_inputs: Vec<PublicInputValue> = vec![
            BlsScalar::from(1).into(),

        ];

        let r = Yao::verify(
            &pp,
            &vd,
            &proof,
            &public_inputs,
            b"Test",
        );

        match r {
            Ok(()) => println!("OK"),
            _  => println!("Wrong")

        }

        t_0 += now.elapsed().as_millis();

    }
    println!("{:?}",t_0);


    sgx_status_t::SGX_SUCCESS
}
