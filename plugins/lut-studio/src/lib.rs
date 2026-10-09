#![no_std]
#[cfg(test)] extern crate std;
use core::slice;
mod lut;
use lut::sample_trilinear;
#[cfg(not(test))] #[panic_handler] fn panic(_: &core::panic::PanicInfo)->!{loop{core::hint::spin_loop();}}
static MANIFEST:&[u8]=br#"{
 "id":"org.photocraft.community.lut-studio","name":"LUT Studio","version":"0.1.0","kind":"filter","author":"ArtCraft Store community",
 "description":"Applies bundled 3D color lookup tables with adjustable intensity.",
 "params":{"preset":{"type":"choice","options":["classic","warm","cool","faded","cinematic"],"default":"classic"},"intensity":{"type":"int","min":0,"max":100,"default":100}},
 "overlap":0,"area":"content"
}"#;
include!(concat!(env!("OUT_DIR"),"/embedded_luts.rs"));
#[no_mangle] pub extern "C" fn pc_abi_version()->i32{1}
#[no_mangle] pub extern "C" fn pc_manifest()->i64{((MANIFEST.len() as i64)<<32)|(MANIFEST.as_ptr() as u32 as i64)}
#[cfg(target_arch="wasm32")] static mut HEAP_NEXT:usize=65536;
#[cfg(target_arch="wasm32")] #[no_mangle] pub extern "C" fn pc_alloc(size:i32)->i32{
 if size<=0{return 0;}let size=size as usize;let start=unsafe{(HEAP_NEXT+7)&!7usize};
 let end=match start.checked_add(size){Some(v)=>v,None=>return 0};let need=end.saturating_add(65535)/65536;
 let now=core::arch::wasm32::memory_size(0);if need>now&&core::arch::wasm32::memory_grow(0,need-now)==usize::MAX{return 0;}
 unsafe{HEAP_NEXT=end;}start as i32
}
#[cfg(not(target_arch="wasm32"))] #[no_mangle] pub extern "C" fn pc_alloc(_:i32)->i32{0}
fn int_param(p:&[u8],key:&[u8],default:i32)->i32{
 let mut i=0;while i+key.len()+2<p.len(){if p[i]==b'"'&&p.get(i+1..i+1+key.len())==Some(key)&&p.get(i+key.len()+1)==Some(&b'"'){
 let mut j=i+key.len()+2;while j<p.len()&&p[j].is_ascii_whitespace(){j+=1;}if p.get(j)!=Some(&b':'){return default;}j+=1;
 while j<p.len()&&p[j].is_ascii_whitespace(){j+=1;}let neg=p.get(j)==Some(&b'-');if neg{j+=1;}let mut n=0i32;let mut found=false;
 while j<p.len()&&p[j].is_ascii_digit(){n=n.saturating_mul(10).saturating_add((p[j]-b'0') as i32);found=true;j+=1;}
 if !found{return default;}return if neg{n.saturating_neg()}else{n};}i+=1;}default
}
fn preset(p:&[u8])->usize{let mut i=0;while i+8<p.len(){if p[i..].starts_with(b"\"preset\""){let rest=&p[i+8..];if let Some(c)=rest.iter().position(|&b|b==b':'){let v=&rest[c+1..];
 if v.windows(7).any(|x|x==b"classic"){return 0;}if v.windows(4).any(|x|x==b"warm"){return 1;}if v.windows(4).any(|x|x==b"cool"){return 2;}if v.windows(5).any(|x|x==b"faded"){return 3;}if v.windows(9).any(|x|x==b"cinematic"){return 4;}}return 0;}i+=1;}0}
fn lut_for(p:usize)->(&'static [[f32;3]],usize){match p{1=>(&LUT_WARM,SIZE_WARM),2=>(&LUT_COOL,SIZE_COOL),3=>(&LUT_FADED,SIZE_FADED),4=>(&LUT_CINEMATIC,SIZE_CINEMATIC),_=>(&LUT_CLASSIC,SIZE_CLASSIC)}}
fn apply(pixel:&mut[f32],colors:usize,alpha:bool,data:&[[f32;3]],size:usize,amount:f32){
 if alpha&&pixel[colors]<=0.0{return;}if amount<=0.0{return;}
 if colors==3{let a=[pixel[0],pixel[1],pixel[2]];let b=sample_trilinear(data,size,[0.0;3],[1.0;3],a);for c in 0..3{pixel[c]=a[c]+(b[c]-a[c])*amount;}}
 else{let a=pixel[0];let b=sample_trilinear(data,size,[0.0;3],[1.0;3],[a;3]);let y=.2126*b[0]+.7152*b[1]+.0722*b[2];pixel[0]=a+(y-a)*amount;}
}
#[no_mangle] pub unsafe extern "C" fn pc_filter(buf:i32,len:i32,w:i32,h:i32,ch:i32,format:i32,params_ptr:i32,params_len:i32)->i32{
 if buf<=0||len<0||w<=0||h<=0||ch<=0||params_ptr<=0||params_len<0{return 1;}
 let count=match (w as usize).checked_mul(h as usize).and_then(|v|v.checked_mul(ch as usize)){Some(v)=>v,None=>return 2};
 if count.checked_mul(4)!=Some(len as usize){return 3;}let mode=(format>>16)&0xff;if !matches!(mode,1|2|3|8){return 5;}
 let channels=ch as usize;let alpha=(format&0x100)!=0&&channels>1;let colors=channels-usize::from(alpha);
 if matches!(mode,2|3)&&colors!=3{return 6;}if matches!(mode,1|8)&&colors!=1{return 6;}
 let p=slice::from_raw_parts(params_ptr as *const u8,params_len as usize);let amount=int_param(p,b"intensity",100).clamp(0,100) as f32/100.0;if amount==0.0{return 0;}
 let (data,size)=lut_for(preset(p));let pixels=slice::from_raw_parts_mut(buf as *mut f32,count);
 for pixel in pixels.chunks_exact_mut(channels){apply(pixel,colors,alpha,data,size,amount);}0
}
#[cfg(test)] mod tests{
 use super::*;
 #[test]fn zero_intensity_is_identity(){let mut p=[0.2,0.4,0.6];let old=p;apply(&mut p,3,false,&LUT_WARM,SIZE_WARM,0.0);assert_eq!(p,old);}
 #[test]fn alpha_preserved_and_transparency_skipped(){let mut p=[0.2,0.4,0.6,0.0];let old=p;apply(&mut p,3,true,&LUT_WARM,SIZE_WARM,1.0);assert_eq!(p,old);let mut q=[0.2,0.4,0.6,0.37];apply(&mut q,3,true,&LUT_WARM,SIZE_WARM,1.0);assert_eq!(q[3],0.37);}
 #[test]fn warm_lut_moves_red_up_blue_down(){let mut p=[0.25,0.4,0.7];apply(&mut p,3,false,&LUT_WARM,SIZE_WARM,1.0);assert!(p[0]>0.25);assert!(p[2]<0.7);}
 #[test]fn presets_differ(){assert_ne!(LUT_WARM[0],LUT_COOL[0]);assert_ne!(LUT_CLASSIC[0],LUT_FADED[0]);assert_eq!(preset(br#"{"preset":"cinematic"}"#),4);}
 #[test]fn grayscale_is_finite(){let mut p=[0.35];apply(&mut p,1,false,&LUT_WARM,SIZE_WARM,1.0);assert!(p[0].is_finite());}
}
