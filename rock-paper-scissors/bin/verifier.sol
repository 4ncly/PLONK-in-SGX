// This file is MIT Licensed.
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
        vk.alpha = Pairing.G1Point(uint256(0x0d789bf24b6d2031d3f71411d166c2687cd122d2b86afd62210460f2b45bc0d9318e9df2d1ec45503f638c09ab28e5fd),uint256(0x07665a746f091cf4e5bfafdf72d6941ec763883fb2d6bd16302045e568d133b5dfc4a2f584dbd04ba9dd289e01e47483));
        vk.beta = Pairing.G2Point([uint256(0x0ef39a5a1e6549de6adb4f6a7bea1b305e13329ae6000e938e0a47724d24a0ca6b5d2f9964bd3eb076b7ecf76c34e47c),uint256(0x113ef412b2c863522d69c09f59f61e2c18e3ea14b8a832c9811a1f78541f01b602603f09f524984cb83e8fdd14a59322)],[uint256(0x0c0838024834d6f737b77b618b7570135c671691707f5b4382ebe1597e79b655f792b6752a0bd0bc12a52e13e2efdf8a),uint256(0x125446f083c68bdd2a8b1f4c3ad4c7cbd3f9be755c8cba2aa1ea75663df663fe1fc1c3f0d653ef5a21aaaca5f8d71249)]);
        vk.gamma = Pairing.G2Point([uint256(0x127b97f09c24e9ea4aa19e8d7bedb0beac0aca93de2f58127616114c763375f3a0aa919f03ca91bb6bc88966fe9af467),uint256(0x15f0bfc629720756f2bab29854509ddddb872a2cf7d2dce6dc6cb8eec75810dc15fe21cab78809d2f04cb226c4c58c88)],[uint256(0x08c0994778fbe3ed367b71d00d96ac90cf16477b11b1130ae3028ef31d0741514392a9596a22d3cdf418ae7f161db50d),uint256(0x198c0a9ac2bcb3025151ed82ba47687c43b10594d467d886375f03b2de9914219de30e4233c0a2a067e1c5b735b1aa91)]);
        vk.delta = Pairing.G2Point([uint256(0x16b913b0172b2eeb3cfb19eebd0376f472df24cef0fa040dfec9ec088ed801df5aa875a4646cc0294354d7bc7d17eab7),uint256(0x0d911eb77e9d0e313a5f341e7adfaf55e6722bb63f4af220eed2f0e0efcde0b1c134d56cebf900b92aa1897f1e72f4ec)],[uint256(0x0320acf95d51f519533c5dee5f512cd5a034a7455df916ee53055970ac1fbb05f180e992d0bac72976de5672bb07598e),uint256(0x177e5d6a3d05f47fc16b312ce0129c479f9aaafb113e10c04da08dbc5005c6bf2a9c44e91a88b135c82f82bf0a16c58d)]);
        vk.gamma_abc = new Pairing.G1Point[](3);
        vk.gamma_abc[0] = Pairing.G1Point(uint256(0x199725ac4c91d817d007d01589cd71c3b9fbea4d9de78f1924bc9846d3d16fefa0febab70515624abb1a494d8b4a7bed),uint256(0x180027ce9e576b70c44589bcdf8f0f15b9c8fde402bdd0c87ed37079e4a199419046a518985f03a45335bde75e09a0d2));
        vk.gamma_abc[1] = Pairing.G1Point(uint256(0x0b9a6a98be1102775230a1874cb3a610489cf84856edbcb901fb5b7604e9d9ac8957445bd9521b8b6b75ac372a4df98d),uint256(0x055d25721640719a79fba956ab357f7146c1e9c4c1a82ff1432d2148961bf70cac92116061968ef610cc326855dc5856));
        vk.gamma_abc[2] = Pairing.G1Point(uint256(0x02937972cb8cf1e48d104aeae58f5beab24aa4df34acc27dd57ca9409ca4fca9abe8356ddc4ebfd6718f98c3e42fe4b1),uint256(0x0998e758e3108d66e4632104b33f27ef0f3a27d2b03d270aad5c942d947f9f9bb57f570dde087a407a140c7fd96c6ede));
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
            Proof memory proof, uint[2] memory input
        ) public view returns (bool r) {
        uint[] memory inputValues = new uint[](2);
        
        for(uint i = 0; i < input.length; i++){
            inputValues[i] = input[i];
        }
        if (verify(inputValues, proof) == 0) {
            return true;
        } else {
            return false;
        }
    }
}
