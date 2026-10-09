#!/usr/bin/env python3
"""Package FilmCraft-native presets and matching 3D LUTs for Film Emulation Toolkit."""
import argparse
import json
import zipfile
from pathlib import Path

SIZE = 17
PRESETS = [
    {"name":"Film Emulation — Classic","description":"Gentle contrast, restrained saturation, subtle monochrome grain.","style":"classic","lumetri":{"contrast":8.0,"saturation":95.0,"faded_film":8.0},"grain":4.0},
    {"name":"Film Emulation — Warm","description":"Warm highlights and subtle monochrome grain.","style":"warm","lumetri":{"temperature":12.0,"tint":3.0,"contrast":6.0,"creative_sat":105.0,"faded_film":5.0,"highlight_tint":[0.9,0.65,0.38,1.0],"shadow_tint":[0.5,0.45,0.38,1.0]},"grain":6.0},
    {"name":"Film Emulation — Cool","description":"Cooler shadows, restrained saturation, subtle monochrome grain.","style":"cool","lumetri":{"temperature":-12.0,"tint":-2.0,"contrast":8.0,"saturation":88.0,"highlight_tint":[0.72,0.78,0.9,1.0],"shadow_tint":[0.36,0.5,0.7,1.0]},"grain":5.0},
    {"name":"Film Emulation — Faded","description":"Lifted blacks, softened contrast, muted colour, fine monochrome grain.","style":"faded","lumetri":{"exposure":0.1,"contrast":-10.0,"saturation":80.0,"faded_film":35.0,"blacks":10.0},"grain":3.0},
    {"name":"Film Emulation — Cinematic","description":"Cool shadows, warm highlights, firmer contrast, subtle monochrome grain.","style":"cinematic","lumetri":{"contrast":15.0,"saturation":90.0,"highlight_tint":[0.72,0.55,0.38,1.0],"shadow_tint":[0.35,0.48,0.58,1.0]},"grain":6.0},
]

def grade(rgb, style):
    r, g, b = rgb
    lum = 0.2126*r + 0.7152*g + 0.0722*b
    if style == "warm":
        r += 0.08*(1-lum); g += 0.025*(1-lum); b -= 0.07*(0.3+lum)
    elif style == "cool":
        r -= 0.06*(0.2+lum); g += 0.01; b += 0.07*(1-lum)
    elif style == "faded":
        r, g, b = r*0.82+0.08, g*0.82+0.08, b*0.82+0.08
    elif style == "cinematic":
        r = (r-0.5)*1.12+0.5+0.04*lum
        g = (g-0.5)*1.12+0.5+0.02*(1-lum)
        b = (b-0.5)*1.12+0.5+0.05*(1-lum)-0.03*lum
    else:
        r = (r-0.5)*1.08+0.5+0.025*(1-lum)
        g = (g-0.5)*1.08+0.5+0.005
        b = (b-0.5)*1.08+0.5-0.02*lum
    lum = 0.2126*r + 0.7152*g + 0.0722*b
    sat = {"classic":0.95,"warm":1.02,"cool":0.88,"faded":0.80,"cinematic":0.90}[style]
    return tuple(max(0.0,min(1.0,lum+(c-lum)*sat)) for c in (r,g,b))

def lut_text(preset):
    lines = ['TITLE "ArtCraft Film Emulation - %s"' % preset["style"].title(),
             "LUT_3D_SIZE %d" % SIZE, "DOMAIN_MIN 0.0 0.0 0.0",
             "DOMAIN_MAX 1.0 1.0 1.0",
             "# Original community look; display-encoded RGB; red changes fastest."]
    n = SIZE - 1
    for b in range(SIZE):
        for g in range(SIZE):
            for r in range(SIZE):
                lines.append("%.7f %.7f %.7f" % grade((r/n,g/n,b/n),preset["style"]))
    result = "\n".join(lines) + "\n"
    assert sum(1 for x in result.splitlines() if x and x[0].isdigit()) == SIZE**3
    return result

def tagged(v):
    if isinstance(v,bool): return {"Bool":v}
    if isinstance(v,(int,float)): return {"Float":float(v)}
    if isinstance(v,list): return {"Color":v}
    raise TypeError(type(v))

def effect(effect_id, values):
    return {"effect":effect_id,"enabled":True,
            "params":{k:{"value":tagged(v)} for k,v in values.items()},
            "masks":[],"post_fader":False,"essential":False,"layer":None}

def presets_json():
    presets = [{"name":p["name"],"description":p["description"],"keyframes":"Scale",
                "source_duration":0,"source_size":[1920,1080],
                "effects":[effect("lumetri",p["lumetri"]),
                           effect("noise",{"amount":p["grain"],"color":False,"clip":True})]}
               for p in PRESETS]
    return json.dumps({"format":"filmcraft.effect-presets","version":1,"presets":presets},indent=2)+"\n"

README = """# Film Emulation Toolkit — FilmCraft pack

This pack complements the PhotoCraft ABI v1 filter with five native FilmCraft effect presets and matching 3D LUTs. FilmCraft does not currently expose a third-party video-effect plug-in ABI, so this is a supported preset/LUT integration rather than a native binary plug-in.

Contents:
- filmcraft-effect-presets.json: five FilmCraft effect-presets v1 presets. Each combines a Lumetri grade with FilmCraft's built-in Noise effect for subtle monochrome grain.
- luts/film-classic.cube, film-warm.cube, film-cool.cube, film-faded.cube, film-cinematic.cube: matching 17-cube LUTs for Lumetri Creative Look. LUTs carry colour only; use the effect presets for grain. Do not stack a matching LUT on its matching grade preset unless you want a stronger look.

Import:
1. Extract this ZIP somewhere permanent.
2. Import the preset JSON using FilmCraft's presets.import command with its full path.
3. Select video clips and apply a Film Emulation preset from the Effect Presets library.
4. Alternatively, import an individual LUT with lut.import and choose it in Lumetri Color > Creative > Look LUT.

CLI example (adjust project and absolute path):
filmcraft-cli --project project.fcproj exec presets.import '{"path":"/absolute/path/filmcraft-effect-presets.json"}'

The presets use only built-in FilmCraft effects (lumetri and noise); they do not install executable code. Check the look on representative footage and adjust grain to taste.
"""

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--output", default="dist/filmcraft-film-emulation-toolkit.zip")
    out = Path(ap.parse_args().output)
    out.parent.mkdir(parents=True, exist_ok=True)
    preset_data = presets_json()
    data = json.loads(preset_data)
    assert data["format"] == "filmcraft.effect-presets" and data["version"] == 1
    assert len(data["presets"]) == 5
    assert all([e["effect"] for e in p["effects"]] == ["lumetri","noise"] for p in data["presets"])
    with zipfile.ZipFile(out,"w",compression=zipfile.ZIP_DEFLATED,compresslevel=9) as z:
        z.writestr("README.md",README)
        z.writestr("filmcraft-effect-presets.json",preset_data)
        for p in PRESETS: z.writestr("luts/film-%s.cube" % p["style"],lut_text(p))
        assert z.testzip() is None
    with zipfile.ZipFile(out) as z:
        names=set(z.namelist())
        assert "filmcraft-effect-presets.json" in names
        for p in PRESETS:
            name="luts/film-%s.cube" % p["style"]
            assert name in names
            text=z.read(name).decode()
            assert "LUT_3D_SIZE %d" % SIZE in text
            assert sum(1 for x in text.splitlines() if x and x[0].isdigit()) == SIZE**3
    print("Built and validated %s (%d bytes); 5 presets and 5 %d-cube LUTs" % (out,out.stat().st_size,SIZE))

if __name__ == "__main__":
    main()
