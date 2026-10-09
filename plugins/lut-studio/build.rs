#[path = "src/lut.rs"] mod lut;
use std::{env,fs,path::PathBuf};
fn main(){
 let names=["classic","warm","cool","faded","cinematic"];let out=PathBuf::from(env::var("OUT_DIR").unwrap());let mut generated=String::new();
 for name in names {let path=format!("luts/{}.cube",name);println!("cargo:rerun-if-changed={}",path);
  let text=fs::read_to_string(&path).expect("read sample .cube");let mut data=vec![[0.0f32;3];33*33*33];
  let info=lut::parse_cube(&text,&mut data).expect("validate sample .cube");let n=info.size*info.size*info.size;
  generated.push_str(&format!("pub static LUT_{}: [[f32;3];{}] = [\n",name.to_uppercase(),n));
  for c in &data[..n]{generated.push_str(&format!("[{:.8}f32,{:.8}f32,{:.8}f32],\n",c[0],c[1],c[2]));}
  generated.push_str("];\n");generated.push_str(&format!("pub const SIZE_{}:usize={};\n",name.to_uppercase(),info.size));
 }
 fs::write(out.join("embedded_luts.rs"),generated).expect("write generated LUT tables");
}
