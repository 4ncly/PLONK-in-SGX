

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