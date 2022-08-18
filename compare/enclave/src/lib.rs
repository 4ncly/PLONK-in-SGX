#![crate_name = "compare"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]


extern crate sgx_types;

#[warn(non_upper_case_globals)]
#[cfg(not(target_env = "sgx"))]
#[macro_use]


extern crate sgx_tstd as std;
extern crate ff;
extern crate bellman;
extern crate rand;
extern crate bls12_381;
extern crate rand_core;
extern crate regex;
extern crate serde_json;


mod circuit;
use circuit::*;
use sgx_types::*;
use std::string::String;
use std::vec::Vec;
use std::fs::OpenOptions;
use rand::thread_rng;
use bls12_381::{Bls12, Scalar};
use bellman::groth16::{
  create_random_proof, 
  generate_random_parameters, 
  prepare_verifying_key, 
  verify_proof,
  Proof,
};
use regex::Regex;
use rand_core::RngCore;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::string::ToString;
use serde_json::{Result, Value};
use ff::Field;
use core::convert::TryInto;

#[no_mangle]
pub extern "C" fn secure_func(some_string: *const u8, some_len: usize) -> sgx_status_t
{
 
    let  enter_string = String::from("******** Enter TEE *********\n");
    println!("{}", &enter_string);

    let mut input = vec![];
    let f = File::open("/root/sgx/mybellman/compare/bin/input.json").unwrap();
    let reader = BufReader::new(f);
    for line in reader.lines()
    {
        let line = line.unwrap();
        let text: &str = &line.to_string()[..];
        let v: Value = serde_json::from_str(&line).unwrap();
        for (k,val) in v.as_object().unwrap().iter()
        {
            input.push(val.as_u64().unwrap());
        }
    }
    println!("got input {:?}",input);


    ZkProof(input);


    let  leave_string = String::from("\n******** Leave TEE *********");
    println!("{}", &leave_string);

    sgx_status_t::SGX_SUCCESS
}

pub fn ZkProof(input: Vec<u64>){

    let mut rng = thread_rng();

    
    let params = 
    {
        let circuit = Compare{a:None, b:None};
        generate_random_parameters::<Bls12, _, _>(circuit, &mut rng).unwrap()
    };
    let pvk = prepare_verifying_key(&params.vk);


    let mut vk_str: Vec<String> = Vec::new();

    let mut s= format!("{:?}",params.vk.alpha_g1);
    vk_str.push(fmt_g1(&s));
    s= format!("{:?}",params.vk.beta_g2);
    vk_str.push(fmt_g2(&s));
    s= format!("{:?}",params.vk.gamma_g2);
    vk_str.push(fmt_g2(&s));
    s= format!("{:?}",params.vk.delta_g2);
    vk_str.push(fmt_g2(&s));

    let gamma_abc_count=params.vk.ic.len();
    vk_str.push(gamma_abc_count.to_string());

    let mut gamma_abc_repeat_text = String::new();
    for (i, g1) in params.vk.ic.iter().enumerate()
    {
        gamma_abc_repeat_text.push_str(
            format!(
                "vk.gamma_abc[{}] = Pairing.G1Point({});",
                i,
                fmt_g1(&format!("{:?}",g1))
            )
            .as_str(),
        );
        if i < gamma_abc_count - 1 {
            gamma_abc_repeat_text.push_str("\n        ");
        }
    }
    vk_str.push(gamma_abc_repeat_text);

    vk_str.push((gamma_abc_count-1).to_string());

    let tmp1 = if gamma_abc_count >1
    {
        r#"
        for(uint i = 0; i < input.length; i++){
            inputValues[i] = input[i];
        }"#
    }
    else{ "" }.to_string();
    
    vk_str.push(tmp1);
    
    let tmp2 = if gamma_abc_count >1
    {
        format!(", uint[{}] memory input", gamma_abc_count - 1)
    }
    else{ "".to_string() };
    vk_str.push(tmp2);

    export_contract(vk_str);



    let mut proof_vec = vec![];
    {

        let a = input[0].try_into().unwrap();
        let b = input[1].try_into().unwrap();
        
        let res = Scalar::from(if a>=b {1} else {0});
        println!("computing result {:?}",res);

        proof_vec.truncate(0);
        {

            let c = Compare {a: Some(a), b: Some(b)};
            let proof = create_random_proof(c, &params, &mut rng).unwrap();

            proof.write(&mut proof_vec).unwrap();
        }

        let proof = Proof::read(&proof_vec[..]).unwrap();
        println!("Generate Proof .... OK!");

        let res = verify_proof(&pvk, &proof, &[res]);
        match res{
            Ok(())  => println!("Verify pass"),
            Err(e) => println!("{:?}",e),
        }
    }
}   

pub fn export_contract(vk :Vec<String>)
{

    let mut f = File::create("verifier.sol").unwrap();
 

    let vk_regex = Regex::new(r#"(<%vk_[^i%]*%>)"#).unwrap();
    let mut template_text = String::from(CONTRACT_TEMPLATE);


    template_text = vk_regex.replace(template_text.as_str(), vk[0].as_str()).into_owned();

    template_text = vk_regex.replace(template_text.as_str(), vk[1].as_str()).into_owned();

    template_text = vk_regex.replace(template_text.as_str(), vk[2].as_str()).into_owned();

    template_text = vk_regex.replace(template_text.as_str(), vk[3].as_str()).into_owned();


    let vk_gamma_abc_len_regex = Regex::new(r#"(<%vk_gamma_abc_length%>)"#).unwrap();

    template_text = vk_gamma_abc_len_regex.replace(template_text.as_str(), vk[4].as_str()).into_owned();



    let vk_gamma_abc_repeat_regex = Regex::new(r#"(<%vk_gamma_abc_pts%>)"#).unwrap();

    template_text = vk_gamma_abc_repeat_regex.replace(template_text.as_str(),vk[5].as_str()).into_owned();


    let vk_input_len_regex = Regex::new(r#"(<%vk_input_length%>)"#).unwrap();

    template_text = vk_input_len_regex.replace(template_text.as_str(),vk[6].as_str()).into_owned();


    let input_loop = Regex::new(r#"(<%input_loop%>)"#).unwrap();

    template_text = input_loop.replace(template_text.as_str(),vk[7].as_str()).into_owned();


    let input_argument = Regex::new(r#"(<%input_argument%>)"#).unwrap();

    
    template_text = input_argument.replace(template_text.as_str(),vk[8].as_str()).into_owned();


    let contract = format!("{} {} {}",pairing_lib_beginning,pairing_lib_ending,template_text);
    f.write(contract.as_bytes()).unwrap();

}


pub fn fmt_g2(s: &str) -> String
{
    let mut res : Vec<String> = Vec::new();
    let re = Regex::new(r"0x[0-9a-fA-F]+").unwrap();

    for cap in re.captures_iter(&s)
    {

        res.push((&cap[0]).to_string());
    }
    format!("[uint256({}),uint256({})],[uint256({}),uint256({})]",res[1],res[0],res[3],res[2])

} 

pub fn fmt_g1(s: &str) -> String
{

    let mut res : Vec<String> = Vec::new();
    let re = Regex::new(r"0x[0-9a-fA-F]+").unwrap();

    for cap in re.captures_iter(&s)
    {

        res.push((&cap[0]).to_string());
    }
    format!("uint256({}),uint256({})",res[1],res[0])

} 

// contract template

const CONTRACT_TEMPLATE: &str = r#"
contract Verifier {
    using Pairing for *;
    struct VerifyingKey {
        Pairing.G1Point alpha;
        Pairing.G2Point beta;
        Pairing.G2Point gamma;
        Pairing.G2Point delta;
        Pairing.G1Point[] gamma_abc;
    }
    struct Proof {
        Pairing.G1Point a;
        Pairing.G2Point b;
        Pairing.G1Point c;
    }
    function verifyingKey() pure internal returns (VerifyingKey memory vk) {
        vk.alpha = Pairing.G1Point(<%vk_alpha%>);
        vk.beta = Pairing.G2Point(<%vk_beta%>);
        vk.gamma = Pairing.G2Point(<%vk_gamma%>);
        vk.delta = Pairing.G2Point(<%vk_delta%>);
        vk.gamma_abc = new Pairing.G1Point[](<%vk_gamma_abc_length%>);
        <%vk_gamma_abc_pts%>
    }
    function verify(uint[] memory input, Proof memory proof) internal view returns (uint) {
        uint256 snark_scalar_field = 21888242871839275222246405745257275088548364400416034343698204186575808495617;
        VerifyingKey memory vk = verifyingKey();
        require(input.length + 1 == vk.gamma_abc.length);
        // Compute the linear combination vk_x
        Pairing.G1Point memory vk_x = Pairing.G1Point(0, 0);
        for (uint i = 0; i < input.length; i++) {
            require(input[i] < snark_scalar_field);
            vk_x = Pairing.addition(vk_x, Pairing.scalar_mul(vk.gamma_abc[i + 1], input[i]));
        }
        vk_x = Pairing.addition(vk_x, vk.gamma_abc[0]);
        if(!Pairing.pairingProd4(
             proof.a, proof.b,
             Pairing.negate(vk_x), vk.gamma,
             Pairing.negate(proof.c), vk.delta,
             Pairing.negate(vk.alpha), vk.beta)) return 1;
        return 0;
    }
    function verifyTx(
            Proof memory proof<%input_argument%>
        ) public view returns (bool r) {
        uint[] memory inputValues = new uint[](<%vk_input_length%>);
        <%input_loop%>
        if (verify(inputValues, proof) == 0) {
            return true;
        } else {
            return false;
        }
    }
}
"#;

const pairing_lib_beginning: &str  = r#"// This file is MIT Licensed.
//
// Copyright 2017 Christian Reitwiessner
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
pragma solidity ^0.8.0;
library Pairing {
    struct G1Point {
        uint X;
        uint Y;
    }
    // Encoding of field elements is: X[0] * z + X[1]
    struct G2Point {
        uint[2] X;
        uint[2] Y;
    }
    /// @return the generator of G1
    function P1() pure internal returns (G1Point memory) {
        return G1Point(1, 2);
    }
    /// @return the generator of G2
    function P2() pure internal returns (G2Point memory) {
        return G2Point(
            [10857046999023057135944570762232829481370756359578518086990519993285655852781,
             11559732032986387107991004021392285783925812861821192530917403151452391805634],
            [8495653923123431417604973247489272438418190587263600148770280649306958101930,
             4082367875863433681332203403145435568316851327593401208105741076214120093531]
        );
    }
    /// @return the negation of p, i.e. p.addition(p.negate()) should be zero.
    function negate(G1Point memory p) pure internal returns (G1Point memory) {
        // The prime q in the base field F_q for G1
        uint q = 21888242871839275222246405745257275088696311157297823662689037894645226208583;
        if (p.X == 0 && p.Y == 0)
            return G1Point(0, 0);
        return G1Point(p.X, q - (p.Y % q));
    }
    /// @return r the sum of two points of G1
    function addition(G1Point memory p1, G1Point memory p2) internal view returns (G1Point memory r) {
        uint[4] memory input;
        input[0] = p1.X;
        input[1] = p1.Y;
        input[2] = p2.X;
        input[3] = p2.Y;
        bool success;
        assembly {
            success := staticcall(sub(gas(), 2000), 6, input, 0xc0, r, 0x60)
            // Use "invalid" to make gas estimation work
            switch success case 0 { invalid() }
        }
        require(success);
    }
"#;


const pairing_lib_ending: &str = r#"
    /// @return r the product of a point on G1 and a scalar, i.e.
    /// p == p.scalar_mul(1) and p.addition(p) == p.scalar_mul(2) for all points p.
    function scalar_mul(G1Point memory p, uint s) internal view returns (G1Point memory r) {
        uint[3] memory input;
        input[0] = p.X;
        input[1] = p.Y;
        input[2] = s;
        bool success;
        assembly {
            success := staticcall(sub(gas(), 2000), 7, input, 0x80, r, 0x60)
            // Use "invalid" to make gas estimation work
            switch success case 0 { invalid() }
        }
        require (success);
    }
    /// @return the result of computing the pairing check
    /// e(p1[0], p2[0]) *  .... * e(p1[n], p2[n]) == 1
    /// For example pairing([P1(), P1().negate()], [P2(), P2()]) should
    /// return true.
    function pairing(G1Point[] memory p1, G2Point[] memory p2) internal view returns (bool) {
        require(p1.length == p2.length);
        uint elements = p1.length;
        uint inputSize = elements * 6;
        uint[] memory input = new uint[](inputSize);
        for (uint i = 0; i < elements; i++)
        {
            input[i * 6 + 0] = p1[i].X;
            input[i * 6 + 1] = p1[i].Y;
            input[i * 6 + 2] = p2[i].X[1];
            input[i * 6 + 3] = p2[i].X[0];
            input[i * 6 + 4] = p2[i].Y[1];
            input[i * 6 + 5] = p2[i].Y[0];
        }
        uint[1] memory out;
        bool success;
        assembly {
            success := staticcall(sub(gas(), 2000), 8, add(input, 0x20), mul(inputSize, 0x20), out, 0x20)
            // Use "invalid" to make gas estimation work
            switch success case 0 { invalid() }
        }
        require(success);
        return out[0] != 0;
    }
    /// Convenience method for a pairing check for two pairs.
    function pairingProd2(G1Point memory a1, G2Point memory a2, G1Point memory b1, G2Point memory b2) internal view returns (bool) {
        G1Point[] memory p1 = new G1Point[](2);
        G2Point[] memory p2 = new G2Point[](2);
        p1[0] = a1;
        p1[1] = b1;
        p2[0] = a2;
        p2[1] = b2;
        return pairing(p1, p2);
    }
    /// Convenience method for a pairing check for three pairs.
    function pairingProd3(
            G1Point memory a1, G2Point memory a2,
            G1Point memory b1, G2Point memory b2,
            G1Point memory c1, G2Point memory c2
    ) internal view returns (bool) {
        G1Point[] memory p1 = new G1Point[](3);
        G2Point[] memory p2 = new G2Point[](3);
        p1[0] = a1;
        p1[1] = b1;
        p1[2] = c1;
        p2[0] = a2;
        p2[1] = b2;
        p2[2] = c2;
        return pairing(p1, p2);
    }
    /// Convenience method for a pairing check for four pairs.
    function pairingProd4(
            G1Point memory a1, G2Point memory a2,
            G1Point memory b1, G2Point memory b2,
            G1Point memory c1, G2Point memory c2,
            G1Point memory d1, G2Point memory d2
    ) internal view returns (bool) {
        G1Point[] memory p1 = new G1Point[](4);
        G2Point[] memory p2 = new G2Point[](4);
        p1[0] = a1;
        p1[1] = b1;
        p1[2] = c1;
        p1[3] = d1;
        p2[0] = a2;
        p2[1] = b2;
        p2[2] = c2;
        p2[3] = d2;
        return pairing(p1, p2);
    }
}
"#;