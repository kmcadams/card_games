use crate::cards::hand::Hand;


#[derive(Clone, Debug)]
pub struct Player{
    name: String,
    pub hand: Hand,
    dealer: bool,
}

impl Player{
    pub fn default() -> Self{
        Player{
            name: "CPU".to_string(),
            hand: Hand::new(),
            dealer: true,

        }
    }

    pub fn new(name: String) -> Self{
        Player{
            hand: Hand::new(),
            dealer: false,
            name
        }
    }

    pub fn get_player_name(&self)->String{
        self.name.clone()
    }

    pub fn reset_hand(&mut self){
        self.hand.clear_hand()
    }

}

