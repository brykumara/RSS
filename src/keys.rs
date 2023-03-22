use std::ops::Add;

use amcl_wrapper::field_elem::FieldElement;
use amcl_wrapper::group_elem::GroupElement;
use amcl_wrapper::group_elem_g2::G2;
use amcl_wrapper::group_elem_g1::G1;

use crate::errors::PSError;
use crate::{VerkeyGroup, SignatureGroup};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sigkey {
    pub x: FieldElement,
    pub y: Vec<FieldElement>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Verkey {
    pub X_tilde: VerkeyGroup,
    pub Y_tilde: Vec<VerkeyGroup>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SKrss {
    pub x: FieldElement,
    pub y: FieldElement,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PKrss {
    pub g: SignatureGroup,
    pub g_tilde: VerkeyGroup,
    pub Y_j_1_to_n: Vec<SignatureGroup>,
    pub Y_k_nplus2_to_2n: Vec<SignatureGroup>,
    pub X_tilde: VerkeyGroup,
    Y_tilde_i: Vec<VerkeyGroup>,
}

// Parameters generated by random oracle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub g: SignatureGroup,
    pub g_tilde: VerkeyGroup,
}

impl Params {
    /// Generate g1, g2. These are shared by signer and all users.
    pub fn new(label: &[u8]) -> Self {
        let g = SignatureGroup::from_msg_hash(&[label, " : g".as_bytes()].concat());
        let g_tilde = VerkeyGroup::from_msg_hash(&[label, " : g_tilde".as_bytes()].concat());
        Self { g, g_tilde }
    }
}

/// Generate signing and verification keys for scheme from 2016 paper
pub fn keygen(count_messages: usize, params: &Params) -> (Sigkey, Verkey) {
    // TODO: Take PRNG as argument
    let x = FieldElement::random();
    let X_tilde = &params.g_tilde * &x;
    let mut y = vec![];
    let mut Y_tilde = vec![];
    for _ in 0..count_messages {
        let y_i = FieldElement::random();
        Y_tilde.push(&params.g_tilde * &y_i);
        y.push(y_i);
    }
    (Sigkey { x, y }, Verkey { X_tilde, Y_tilde })
}

pub fn rsskeygen(count_messages: usize, params: &Params) -> (SKrss, PKrss) {
    let x = FieldElement::random(); // x
    let y = FieldElement::random(); // y
    let X_tilde = params.g_tilde.scalar_mul_const_time(&x); // g~ * x
    
    let g = params.g.scalar_mul_variable_time(&FieldElement::one());
    let g_tilde= params.g_tilde.scalar_mul_variable_time(&FieldElement::one());

    let mut Y_tilde_i:Vec<VerkeyGroup> = vec![]; // Create a vector to store Y~i
    let mut i_exponent = FieldElement::one(); // start of exponent
    
    for _ in 0..count_messages{
        let y_i=
        FieldElement::pow(&y,&i_exponent); // Calculate y ^ i 
        
        let g_tilde_y_i = 
        params.g_tilde.scalar_mul_variable_time(&y_i); // Calculate g_tilde * y^i
        
        Y_tilde_i.push(g_tilde_y_i); // Add g_tilde * y^i to Y_tilde_i

        let one = FieldElement::one(); // create counter to increment 
        let i_exponent = 
        FieldElement::add_assign_(&mut i_exponent, &one); //increment i by 1
    }
    
    let mut  Y_j_1_to_n:Vec<G2> = vec![]; // Create a vector to store Y_i
    
    for _ in 0..count_messages{
        let y_i=
        FieldElement::pow(&y,&i_exponent); // Calculate y^i 
        
        let g_y_i = 
        params.g.scalar_mul_variable_time(&y_i); // Calculate g_tilde^y^i
        
        Y_j_1_to_n.push(g_y_i); // Add g_tilde^y^i to Y_tilde_i
        
        let one = FieldElement::one(); // create counter to increment 
        let i_exponent = 
        FieldElement::add_assign_(&mut i_exponent, &one); //increment i by 1
    }
   
    let mut  Y_k_nplus2_to_2n:Vec<G2> = vec![];
    let mut k_exponent = FieldElement::one(); 
    for _ in (count_messages+2)..(2*count_messages) {
        let y_i=FieldElement::pow(&y,&k_exponent); // Calculate y^i
        let g_y_i = params.g.scalar_mul_variable_time(&y_i);
        let y_i = FieldElement::random();
        Y_k_nplus2_to_2n.push(g_y_i);
        let one = FieldElement::one(); // create counter to increment 
        let k_exponent = FieldElement::add_assign_(&mut k_exponent, &one);
    }
   (SKrss {x , y}, PKrss {g , g_tilde , Y_j_1_to_n , Y_k_nplus2_to_2n , X_tilde , Y_tilde_i})
}


/// Generate signing and verification keys for scheme from 2018 paper. The signing and verification
/// keys will have 1 extra element for m'
pub fn keygen_2018(count_messages: usize, params: &Params) -> (Sigkey, Verkey) {
    keygen(count_messages + 1, params)
}

#[cfg(test)]
mod tests {
    use super::*;
    // For benchmarking
    use std::time::{Duration, Instant};
    #[test]
    fn test_keygen() {
        let count_msgs = 5;
        let params = Params::new("test".as_bytes());
        let (sk, vk) = keygen(count_msgs, &params);
        assert_eq!(sk.y.len(), count_msgs);
        assert_eq!(vk.Y_tilde.len(), count_msgs);
    }

    #[test]
    fn test_keygen_2018() {
        let count_msgs = 5;
        let params = Params::new("test".as_bytes());
        let (sk, vk) = keygen_2018(count_msgs, &params);
        assert_eq!(sk.y.len(), count_msgs+1);
        assert_eq!(vk.Y_tilde.len(), count_msgs+1);
    }
    #[test]
    fn test_rsskeygen() {
        let count_msgs = 5;
        let params = Params::new("test".as_bytes());
        let (sk, pk) = rsskeygen(count_msgs, &params);
        println!("{:?}",sk);
        println!("{:?}",pk);
    }

}
