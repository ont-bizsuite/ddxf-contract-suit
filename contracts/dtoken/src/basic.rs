use super::database::*;
use super::ostd::abi::{Decoder, Encoder, Error, Sink, Source};
use super::ostd::types::{Address, U128};
use super::BTreeMap;

//#[derive(Clone)]
//pub struct TokenTemplates<'a> {
//    pub val: BTreeMap<&'a [u8], bool>,
//}
//
//impl<'a> TokenTemplates<'a> {
//    pub fn new(token_hash: &'a [u8]) -> Self {
//        let mut val: BTreeMap<&[u8], bool> = BTreeMap::new();
//        val.insert(token_hash, true);
//        TokenTemplates { val }
//    }
//}
//
//impl<'a> Encoder for TokenTemplates<'a> {
//    fn encode(&self, sink: &mut Sink) {
//        let l = self.val.len() as u32;
//        sink.write(l);
//        for (k, v) in self.val.iter() {
//            sink.write(k);
//            sink.write(v);
//        }
//    }
//}
//
//impl<'a> Decoder<'a> for TokenTemplates<'a> {
//    fn decode(source: &mut Source<'a>) -> Result<Self, Error> {
//        let l: u32 = source.read()?;
//        let mut bmap: BTreeMap<&[u8], bool> = BTreeMap::new();
//        for _ in 0..l {
//            let (k, v) = source.read()?;
//            bmap.insert(k, v);
//        }
//        Ok(TokenTemplates { val: bmap })
//    }
//}

#[derive(Clone)]
pub struct CountAndAgent {
    pub count: u32,
    pub agents: BTreeMap<Address, u32>,
}

impl CountAndAgent {
    pub fn new(addr: Address) -> Self {
        let agents: BTreeMap<Address, u32> = BTreeMap::new();
        CountAndAgent { count: 0, agents }
    }

    pub fn add_agents(&mut self, agents: &[Address], n: u32) {
        for &agent in agents {
            let count = self.agents.entry(agent).or_insert(0);
            *count += n;
        }
    }
    pub fn remove_agents(&mut self, agents: &[Address]) {
        for agent in agents {
            self.agents.remove(agent);
        }
    }
    pub fn set_token_agents(&mut self, agents: &[Address], n: U128) {
        self.agents.clear();
        for &agent in agents.iter() {
            self.agents.insert(agent, n as u32);
        }
    }
}

impl Encoder for CountAndAgent {
    fn encode(&self, sink: &mut Sink) {
        sink.write(self.count);
        let l = self.agents.len() as u32;
        sink.write(l);
        for (k, v) in self.agents.iter() {
            sink.write(k);
            sink.write(v);
        }
    }
}

impl<'a> Decoder<'a> for CountAndAgent {
    fn decode(source: &mut Source<'a>) -> Result<Self, Error> {
        let count: u32 = source.read()?;
        let l: u32 = source.read()?;
        let mut agents: BTreeMap<Address, u32> = BTreeMap::new();
        for _ in 0..l {
            let (k, v): (Address, u32) = source.read()?;
            (&mut agents).insert(k, v);
        }
        Ok(CountAndAgent { count, agents })
    }
}
