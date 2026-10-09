#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubeInfo { pub size: usize, pub domain_min: [f32;3], pub domain_max: [f32;3] }
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CubeError { MissingSize, InvalidSize, Unsupported1D, InvalidDirective, InvalidNumber, InvalidDomain, BufferTooSmall, WrongEntryCount }

fn triplet(v: &[&str]) -> Result<[f32;3], CubeError> {
    if v.len()!=3 { return Err(CubeError::InvalidNumber); }
    let mut out=[0.0;3];
    for i in 0..3 { out[i]=v[i].parse::<f32>().map_err(|_|CubeError::InvalidNumber)?; if !out[i].is_finite(){return Err(CubeError::InvalidNumber);} }
    Ok(out)
}

/// Parses a standard 3D .cube file into caller-owned storage. Red varies fastest.
pub fn parse_cube(input:&str, output:&mut [[f32;3]]) -> Result<CubeInfo,CubeError> {
    let mut size=0usize; let mut min=[0.0,0.0,0.0]; let mut max=[1.0,1.0,1.0]; let mut count=0usize;
    for raw in input.lines() {
        let line=raw.split('#').next().unwrap_or("").trim(); if line.is_empty(){continue;}
        let mut p=line.split_whitespace(); let key=p.next().ok_or(CubeError::InvalidDirective)?;
        match key {
            "TITLE"=>continue,
            "LUT_1D_SIZE"=>return Err(CubeError::Unsupported1D),
            "LUT_3D_SIZE"=>{ let s=p.next().ok_or(CubeError::InvalidSize)?; if p.next().is_some(){return Err(CubeError::InvalidSize);}
                size=s.parse::<usize>().map_err(|_|CubeError::InvalidSize)?; if !(2..=33).contains(&size){return Err(CubeError::InvalidSize);}
                if size*size*size>output.len(){return Err(CubeError::BufferTooSmall);} }
            "DOMAIN_MIN"|"DOMAIN_MAX"=>{ let a=[p.next().ok_or(CubeError::InvalidDomain)?,p.next().ok_or(CubeError::InvalidDomain)?,p.next().ok_or(CubeError::InvalidDomain)?];
                if p.next().is_some(){return Err(CubeError::InvalidDomain);} let v=triplet(&a).map_err(|_|CubeError::InvalidDomain)?;
                if key=="DOMAIN_MIN"{min=v}else{max=v} }
            _=>{ if size==0{return Err(CubeError::MissingSize);}
                let a=[key,p.next().ok_or(CubeError::InvalidNumber)?,p.next().ok_or(CubeError::InvalidNumber)?];
                if p.next().is_some(){return Err(CubeError::InvalidNumber);} if count>=size*size*size{return Err(CubeError::WrongEntryCount);}
                output[count]=triplet(&a)?; count+=1; }
        }
    }
    if size==0{return Err(CubeError::MissingSize);}
    for i in 0..3 {if max[i]<=min[i]{return Err(CubeError::InvalidDomain);}}
    if count!=size*size*size{return Err(CubeError::WrongEntryCount);}
    Ok(CubeInfo{size,domain_min:min,domain_max:max})
}

/// Trilinear LUT sampling, with input values normalized to the LUT's declared domain.
pub fn sample_trilinear(data:&[[f32;3]],size:usize,min:[f32;3],max:[f32;3],rgb:[f32;3])->[f32;3] {
    let mut base=[0usize;3]; let mut f=[0.0;3];
    for a in 0..3 { let pos=(((rgb[a]-min[a])/(max[a]-min[a])).clamp(0.0,1.0))*(size-1) as f32;
        base[a]=(pos as usize).min(size-1); f[a]=if base[a]+1>=size{0.0}else{pos-base[a] as f32}; }
    let [r,g,b]=base; let r1=(r+1).min(size-1);let g1=(g+1).min(size-1);let b1=(b+1).min(size-1);
    let at=|x:usize,y:usize,z:usize|data[(z*size+y)*size+x];
    let c000=at(r,g,b);let c100=at(r1,g,b);let c010=at(r,g1,b);let c110=at(r1,g1,b);
    let c001=at(r,g,b1);let c101=at(r1,g,b1);let c011=at(r,g1,b1);let c111=at(r1,g1,b1);
    let mut out=[0.0;3];
    for c in 0..3 {let x00=c000[c]+(c100[c]-c000[c])*f[0];let x10=c010[c]+(c110[c]-c010[c])*f[0];
        let x01=c001[c]+(c101[c]-c001[c])*f[0];let x11=c011[c]+(c111[c]-c011[c])*f[0];
        let y0=x00+(x10-x00)*f[1];let y1=x01+(x11-x01)*f[1];out[c]=y0+(y1-y0)*f[2];}
    out
}

#[cfg(test)]
mod tests {
 use super::*;
 const ID:&str="TITLE \"Identity\"\nLUT_3D_SIZE 2\nDOMAIN_MIN 0 0 0\nDOMAIN_MAX 1 1 1\n0 0 0\n1 0 0\n0 1 0\n1 1 0\n0 0 1\n1 0 1\n0 1 1\n1 1 1\n";
 #[test] fn parses_cube_order(){let mut d=[[0.0;3];8];let i=parse_cube(ID,&mut d).unwrap();assert_eq!(i.size,2);assert_eq!(d[1],[1.0,0.0,0.0]);assert_eq!(d[4],[0.0,0.0,1.0]);}
 #[test] fn identity_interpolates(){let mut d=[[0.0;3];8];let i=parse_cube(ID,&mut d).unwrap();let o=sample_trilinear(&d,i.size,i.domain_min,i.domain_max,[0.2,0.4,0.8]);for c in 0..3{assert!((o[c]-[0.2,0.4,0.8][c]).abs()<0.0001);}}
 #[test] fn rejects_1d(){let mut d=[[0.0;3];8];assert_eq!(parse_cube("LUT_1D_SIZE 16\n",&mut d),Err(CubeError::Unsupported1D));}
 #[test] fn rejects_incomplete(){let mut d=[[0.0;3];8];assert_eq!(parse_cube("LUT_3D_SIZE 2\n0 0 0\n",&mut d),Err(CubeError::WrongEntryCount));}
 #[test] fn rejects_bad_domain(){let mut d=[[0.0;3];8];let s=ID.replace("DOMAIN_MAX 1 1 1","DOMAIN_MAX 0 1 1");assert_eq!(parse_cube(&s,&mut d),Err(CubeError::InvalidDomain));}
}
