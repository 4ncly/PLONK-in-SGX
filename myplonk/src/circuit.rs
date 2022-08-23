

use dusk_plonk::error::Error;
use dusk_bls12_381::BlsScalar;
use dusk_plonk::prelude::*;
// Implement a circuit that checks:
// 1) a + b = c where C is a PI
// 2) a <= 2^6
// 3) b <= 2^5
// 4) a * b = d where D is a PI
// 5) JubJub::GENERATOR * e(JubJubScalar) = f where F is a Public Input

#[derive(Debug, Default)]
pub struct TestCircuit{
    pub a: BlsScalar,
    pub b: BlsScalar,
    pub c: BlsScalar
}


impl Circuit for TestCircuit{
    const CIRCUIT_ID: [u8; 32] = [0xff; 32];

    fn gadget(&mut self,composer: &mut TurboComposer,) -> Result<(), Error>
    {
        
        let a = composer.append_witness(self.a);
        let b = composer.append_witness(self.b);
        let constraint = Constraint::new().left(1).right(1).public(-self.c).a(a).b(b);
    
        composer.append_gate(constraint);

        Ok(())
    }

    fn public_inputs(&self) -> Vec<PublicInputValue> {
        vec![self.c.into()]
    }

    fn padded_gates(&self) -> usize {
        1 << 11
    }
}


#[derive(Debug, Default)]
pub struct Sudoku 
{
    pub answer: [BlsScalar; 4]
}


impl Circuit for Sudoku{
    
    const CIRCUIT_ID: [u8; 32] = [0xff;32];

    fn gadget(&mut self, composer: &mut dusk_plonk::constraint_system::TurboComposer) -> core::result::Result<(), dusk_plonk::error::Error> { 

        let one = BlsScalar::from(1);

        let witness_zero =  composer.append_witness(BlsScalar::from(0));
        let witness_one =  composer.append_witness(BlsScalar::from(1));
        let witness_two =  composer.append_witness(BlsScalar::from(2));
        let witness_four =  composer.append_witness(BlsScalar::from(4));
        let witness_eight =  composer.append_witness(BlsScalar::from(8));

        let tmp =  composer.append_witness(BlsScalar::from(0));

        let r0 = conditionEq(self.answer[0], BlsScalar::from(1), composer);
        let add0 = composer.component_select_0(r0,witness_one);
        let c0 = Constraint::new().left(1).right(1).a(tmp).b(add0);
        let tmp = composer.gate_add(c0);

        let r1 = conditionEq(self.answer[0], BlsScalar::from(2), composer);
        let add1 = composer.component_select_0(r1,witness_two);
        let c1 = Constraint::new().left(1).right(1).a(tmp).b(add1);
        let tmp = composer.gate_add(c1);

        let r2 = conditionEq(self.answer[0], BlsScalar::from(4), composer);
        let add2 = composer.component_select_0(r2,witness_four);
        let c2 = Constraint::new().left(1).right(1).a(tmp).b(add2);
        let tmp = composer.gate_add(c2);

        let r3 = conditionEq(self.answer[0], BlsScalar::from(7), composer);
        let add3 = composer.component_select_0(r3,witness_eight);
        let c3 = Constraint::new().left(1).right(1).a(tmp).b(add3);
        let tmp = composer.gate_add(c3);
        

        let r0 = conditionEq(self.answer[1], BlsScalar::from(1), composer);
        let add0 = composer.component_select_0(r0,witness_one);
        let c0 = Constraint::new().left(1).right(1).a(tmp).b(add0);
        let tmp = composer.gate_add(c0);

        let r1 = conditionEq(self.answer[1], BlsScalar::from(2), composer);
        let add1 = composer.component_select_0(r1,witness_two);
        let c1 = Constraint::new().left(1).right(1).a(tmp).b(add1);
        let tmp = composer.gate_add(c1);

        let r2 = conditionEq(self.answer[1], BlsScalar::from(4), composer);
        let add2 = composer.component_select_0(r2,witness_four);
        let c2 = Constraint::new().left(1).right(1).a(tmp).b(add2);
        let tmp = composer.gate_add(c2);

        let r3 = conditionEq(self.answer[1], BlsScalar::from(7), composer);
        let add3 = composer.component_select_0(r3,witness_eight);
        let c3 = Constraint::new().left(1).right(1).a(tmp).b(add3);
        let tmp = composer.gate_add(c3);

        let r0 = conditionEq(self.answer[2], BlsScalar::from(1), composer);
        let add0 = composer.component_select_0(r0,witness_one);
        let c0 = Constraint::new().left(1).right(1).a(tmp).b(add0);
        let tmp = composer.gate_add(c0);

        let r1 = conditionEq(self.answer[2], BlsScalar::from(2), composer);
        let add1 = composer.component_select_0(r1,witness_two);
        let c1 = Constraint::new().left(1).right(1).a(tmp).b(add1);
        let tmp = composer.gate_add(c1);

        let r2 = conditionEq(self.answer[2], BlsScalar::from(4), composer);
        let add2 = composer.component_select_0(r2,witness_four);
        let c2 = Constraint::new().left(1).right(1).a(tmp).b(add2);
        let tmp = composer.gate_add(c2);

        let r3 = conditionEq(self.answer[2], BlsScalar::from(7), composer);
        let add3 = composer.component_select_0(r3,witness_eight);
        let c3 = Constraint::new().left(1).right(1).a(tmp).b(add3);
        let tmp = composer.gate_add(c3);


        let r0 = conditionEq(self.answer[3], BlsScalar::from(1), composer);
        let add0 = composer.component_select_0(r0,witness_one);
        let c0 = Constraint::new().left(1).right(1).a(tmp).b(add0);
        let tmp = composer.gate_add(c0);

        let r1 = conditionEq(self.answer[3], BlsScalar::from(2), composer);
        let add1 = composer.component_select_0(r1,witness_two);
        let c1 = Constraint::new().left(1).right(1).a(tmp).b(add1);
        let tmp = composer.gate_add(c1);

        let r2 = conditionEq(self.answer[3], BlsScalar::from(4), composer);
        let add2 = composer.component_select_0(r2,witness_four);
        let c2 = Constraint::new().left(1).right(1).a(tmp).b(add2);
        let tmp = composer.gate_add(c2);

        let r3 = conditionEq(self.answer[3], BlsScalar::from(7), composer);
        let add3 = composer.component_select_0(r3,witness_eight);
        let c3 = Constraint::new().left(1).right(1).a(tmp).b(add3);
        let tmp = composer.gate_add(c3);





        composer.assert_equal_constant(tmp,BlsScalar::from(15),None);

        // println!("{:?}",composer.n);

        Ok(())
    }
    fn public_inputs(&self) -> std::vec::Vec<dusk_plonk::circuit::PublicInputValue> {

        vec![]
    }
    fn padded_gates(&self) -> usize { 
        1 << 11
    }
}

fn conditionEq(a:BlsScalar, b:BlsScalar,composer: &mut dusk_plonk::constraint_system::TurboComposer) -> dusk_plonk::constraint_system::Witness
{
    let one = BlsScalar::from(1);
    let x0;
    let mut y0;
    if a==b {
        x0=BlsScalar::from(0);
        y0=BlsScalar::from(1);
    }
    else{
        x0=BlsScalar::from(1);
        y0=a-b;
        y0=y0.invert().unwrap();
    }

    let witness_x0 = composer.append_witness(x0);
    let witness_y0 = composer.append_witness(y0);
    // let witness_one = composer.append_witness(one);
    let _0 = composer.append_witness(a-b);

    let constraint1 = Constraint::new().a(_0).b(witness_y0).mult(1).o(witness_x0).output(-one);
    let constraint2 = Constraint::new().left(-one).a(_0).b(witness_x0).mult(1);

    // println!("{:?}",x0);

    composer.append_gate(constraint1);
    composer.append_gate(constraint2);
    witness_x0

}


#[derive(Debug, Default)]
pub struct Yao 
{
    pub a: BlsScalar,
    pub b: BlsScalar,
    pub c: BlsScalar
}


impl Circuit for Yao{
    
    const CIRCUIT_ID: [u8; 32] = [0xff;32];
    fn gadget(&mut self, composer: &mut dusk_plonk::constraint_system::TurboComposer) -> core::result::Result<(), dusk_plonk::error::Error> { 

        let a = composer.append_witness(self.a);
        let b = composer.append_witness(self.b);

        composer.component_range(a, 1 << 8);
        composer.component_range(b, 1 << 8);



        let sub = composer.append_witness(self.a+BlsScalar::from(256)-self.b);
        let bits_sub = composer.component_decomposition::<9>(sub);

        composer.assert_equal_constant(bits_sub[0],BlsScalar::from(self.c),None);


        Ok(())

    }
    fn public_inputs(&self) -> std::vec::Vec<dusk_plonk::circuit::PublicInputValue> { 
        
        vec![self.c.into()]

    }
    fn padded_gates(&self) -> usize { 
        1 << 11
    }
}