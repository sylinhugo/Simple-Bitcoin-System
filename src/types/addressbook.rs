
use serde::{Deserialize, Serialize};
// use std::convert::TryInto;
// use std::collections::{HashMap, HashSet};
// use std::vec;
// Add ring crate
// use ring::digest::{self, Context, Digest, SHA256};



#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Hash, Default, Copy)]
pub struct AddressBook {

}


impl AddressBook {
    pub fn new () ->Self {
        Self {}
    }

    pub fn search(&mut self, index: u32) -> Vec<Vec<String>>{ 
        
        
        if index == 0{
            let mut res = Vec::new();
            let mut vector0 = Vec::new();
            vector0.push("0000000000000000000000000000000000000000000000000000000000000000 0 10000 24a19420618b8aba7bdf30949aa8ec1658c54607".to_string());
            res.push(vector0);
            return res.clone();
        }
        



        if index <= 10{
            let mut res2 = Vec::new();
            let mut vector10 = Vec::new();
            vector10.push("009a2b974133e5e3d0e8081741f85cf55240eabba4f2b39faa9926b3fc93dc8a 0 9970 24a19420618b8aba7bdf30949aa8ec1658c54607".to_string());
            vector10.push("01fd0f9565b0d185b076f76061f230ec3c06a3ab5ba2ef5eeaf5e1a987a620ab 0 15 185b76ddadce60ab837a48c5cacbca47fdba63d6".to_string());
            vector10.push("02d26e9b17926f287e9c5c0ea1704b120cf05f25501d1e05904c1b44c03ae0e1 0 4 b4101356b8d0a5d36c7a8d2fc679bc3d0314dc3c".to_string());
            vector10.push("040382c5f0ea00d65ed8466c01fcea8bbdb13d184d0c3b74e2c48522dbc792fd 0 7 b4101356b8d0a5d36c7a8d2fc679bc3d0314dc3c".to_string());
            vector10.push("04ec390e8b0653a944f5e33c7f9eb51d10dcf76d70997eafceb9c5ffca00009b 0 4 24a19420618b8aba7bdf30949aa8ec1658c54607".to_string());
            res2.push(vector10);
            return res2.clone();
        }

        let mut res3 = Vec::new();
        let mut vector20 = Vec::new();
        vector20.push("10ede0fee6e075da9b15c136ff21cc43e92643fddf997dd99da67fac56a4bb4a 0 9934 24a19420618b8aba7bdf30949aa8ec1658c54607".to_string());
        vector20.push("1301ae5ff3352dd1b8a7de962aa5ceb7418232a2304cfe91037693e0758a7bc8 0 13 b4101356b8d0a5d36c7a8d2fc679bc3d0314dc3c".to_string());
        vector20.push("14cfba60fc80ec36a73b0008b9b1c26f1e3db2f0b5d11385f4ec5fe35fdc2efb 0 6 185b76ddadce60ab837a48c5cacbca47fdba63d6".to_string());
        vector20.push("15af40eb9a91d9387e097793d624007c71e73c7efab8bedcc1b6e93a369885c4 0 16 b4101356b8d0a5d36c7a8d2fc679bc3d0314dc3c".to_string());
        vector20.push("1886a1a3cf6d24f274f118d4b0b9ecc69749aba85c73cc3882a03b05e07060ca 0 8 185b76ddadce60ab837a48c5cacbca47fdba63d6".to_string());
        vector20.push("1a86b1628cf308c4c933157a097fefe51927e63fde1e7c0f0957055db184073a 0 2 24a19420618b8aba7bdf30949aa8ec1658c54607".to_string());
        vector20.push("20362e7842bd871204e851c52528275bf13eec2f81ea8532dfa888fd718ceca5 0 10 24a19420618b8aba7bdf30949aa8ec1658c54607".to_string());
        vector20.push("222f7d5e8de018731cada1c8a55ece2cd5b6e38465c38f64bd410b5ffd34db70 0 11 b4101356b8d0a5d36c7a8d2fc679bc3d0314dc3c".to_string());
        res3.push(vector20);
        return res3;

    }
}

