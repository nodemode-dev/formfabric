# Marvelous Designer (MD): Import/Export, UV, Retopology, Baking, Python API and Pipeline Integrations

Researcher slice 09 of 13. Purpose: feed the formfabric open-source clone spec.
Research date: 2026-11-01. Product versions covered: MD 2024.x to 2026.1 (current docs), with legacy version tags as given by the manual.

## Summary

- MD exports are all driven by one shared "Export" dialog family. The same core knobs recur in OBJ, FBX, Alembic, USD, glTF, cache (MDD/PC2/MC) and OBJ sequence: object selection, single vs multiple objects, weld mode (3 modes), thin vs thick, unified UV, scale, axis conversion, animation range. A clone should implement ONE options struct and per-format subsets (this is exactly what MD's Python API does with `ImportExportOption`).
- Thick export is only valid when the pattern mesh is Quad (manual, ver 2.3.0 note). Default recommended mesh is Triangle; Quad is for smooth rounding (collars, piping). Newer mesh styles: Triangle/Quad (Optimized) (2024.0) and Quad (Grid) (2024.0/2025.0).
- Unified UV is a pack-all-patterns-into-0..1 export, with baking of textures (diffuse, normal, metalness, roughness, opacity, displacement) with size, seam fill (padding), "infinite seams", and UDIM-like tile selection (0-1, selected tile by U,V, or all tiles).
- Retopology in MD is manual: a Topology tool and Edit Topology tool draw 3/4-point faces on patterns, with remesh (duplicate/replace) commands, ghost-point snapping across seams, and a Patch Topology tool. There is also an automatic "Remeshing" command that produces an aligned quad grid. No exposed numeric settings for auto-remesh other than particle distance.
- Live pipeline: LiveSync plugin for Unreal Engine (CLO/MD LiveSync 2.x), plus a USD export "Simulation Data" path for Chaos Cloth and MetaHuman (UE 5.4 with MD 2025.0, UE 5.6+ with MD 2025.2, UE 5.5 incompatible for ChaosCloth).
- Python API: embedded Python editor, a module-based API (EXPORT_API, IMPORT_API, FABRIC_API, PATTERN_API, UTILITY_API), Windows, macOS and (since 2025-09-30) Linux (Rocky 9 / RHEL 9).
- Several things asked about were NOT found in official docs and are marked "Not verified": a dedicated Collada (.dae) export, PLY export, DXF/AI import details, UDIM naming conventions, unwrap/seam-cutting tools for 3D UVs, AO baking, LOD generation, GoZ, Blender/Maya/3ds Max/Houdini/C4D/Daz/Character Creator/Unity/iClone official plugins, and "MD Cloud".

## Confidence legend

- [single source] one approved primary source only.
- [verified x2] confirmed by two approved pages (for example the manual page plus the developer API page).
- [not verified] could not be confirmed from primary sources in this session.

## 1. Supported file format matrix (official "Compatible File Format" page)

| Format | Ext | Import | Export | Notes |
|---|---|---|---|---|
| FBX | .fbx | Yes | Yes | Mesh, material, joint, animation, camera. Only joints that were imported via FBX can be exported as FBX. |
| OpenCOLLADA | .dae | Yes | No | Geometry, materials, rigging, skinning, animation. 3ds Max / Maya need OpenCOLLADA plugin to author. |
| OBJ | .obj | Yes | Yes | Geometry and materials; mainly avatars on import. |
| LXO | .lxo | No | Yes | Export of apparel and avatar to Modo. |
| Animation cache | .mc, .pc2, .mdd | Yes | Yes | Vertex animation. Moves to 3ds Max / Maya as point caches. |
| Alembic | .abc | Yes | Yes | Interchange; vertex and animation data. |
| 3D snapshot | .png | N/A | Yes | Single views of the 3D window. |
| 2D texture | .jpg .png etc | Yes | Yes | JPG, PNG, TIF, PSD etc. |
| Substance | .sbsar | Yes | No | Procedural; color, normal, metalness, roughness, opacity can be baked and exported. |

Additional formats from other official pages:

| Format | Where | Notes |
|---|---|---|
| glTF / GLB (glTF 2.0) | MD 2026.0 manual page | Import and export both documented. |
| USD | USD Import/Export page | Import and export, with simulation data export for Chaos Cloth. |
| OBJ sequence | OBJ Sequence Export page | Frame-per-file OBJ export. |
| ZPAC, ZPRJ | File format page | Native garment and project formats. |

Collada: only IMPORT is documented. Collada export: [not verified] and the compatibility table says No. PLY: [not verified], not listed anywhere in official docs found.

## 2. Native file formats (MD file format page)

| Format | Ext | Stores |
|---|---|---|
| Garment | .zpac | Garment, pattern, sewing, fabric data |
| Project | .zprj | Bundle: avatar (AVT), garment (ZPac), any animation |
| Avatar | .avt | Avatar model, sizing, poses, motions, accessories, bounding volumes, arrangement points |
| Avatar encrypted | .avte | Encrypted MD/CLO avatars incl. MetaHuman body types; cannot be exported |
| Pose | .pos | Avatar pose, auto-saved with the garment; poses from other software need a COLLADA avatar with joints |
| Joint motion | .mtn | Avatar motion; requires COLLADA avatar with joints and animation |
| Fabric | .zfab | Fabric texture and physical property values |
| Physical property | .psp | Detailed fabric physical values |
| Topstitch | .sst | Stitch type, line count, thread thickness, SPI, length |
| Button | .btn | Type, size, color |
| Buttonhole | .bth | Style, size, color |
| Arrangement point | .arr | Blue points around the avatar to place patterns |
| Arrangement bounding volume | .pan | Green cylinders covering body parts, ends tied to joints |
| Tape measure | .mea | Tape measure data on avatar |
| Camera projection | .cmp | Field of view, near/far |
| Camera transformation | .cmt | 3D view camera |
| Simulation property | .smp | Simulation settings |

The article does not list DXF or AI as formats. Pattern CPX import/export exists in the API (`ExportPatternCPX`, `ImportPatternCPX`, MD 2026.1 docs) and JSON pattern import/export is deprecated and returns False.

### 2.1 ZPRJ / ZPAC with metadata

Menu: Save As Project with Metadata / Garment with Meta Data. Produces an XML file and turntable images (needs ver 3.1.38+).

| XML content | Meaning |
|---|---|
| Product version, garment version | Versioning |
| Garment counts | polygons, triangles, vertices (patterns, trims, textures) |
| Pattern, trim, button counts | Trim = objects imported via Import as Trim |
| Print texture count | Textures imported as Graphic |
| Texture count, material count | Textures applied to Fabric, items used in Fabric |
| Avatar counts | total polygons, triangles, vertices |
| Turntable image names | |

Save Metadata dialog: File Save Folder (project, metadata, images in a folder), Save .zip File (same in archive), Image Count (number of turntable images), Don't Show Again (reset under User Settings > Others > Metadata).

### 2.2 ZPRJ load options (API `ImportZPRJOption`)

| Field | Meaning |
|---|---|
| bAppend | Add to current scene vs replace |
| bLoadGarment, bLoadAvatar | Which parts to load |
| bLoadSceneAndProps | Load scene and props |
| bLoadRenderProperties | Load render settings |
| bLoadCustomView | Default false |
| bLoadDisplaySettings | Default true |
| translationValueX/Y/Z, translationUnit | Offset applied on load; unit enum 0 mm, 1 cm, 2 inch, 3 m, 4 feet |

## 3. Export dialogs, per format

### 3.1 OBJ export (File > Export > OBJ, or OBJ (Selected))

Writes .obj plus .mtl for garments.

| Option | Meaning | Version tag |
|---|---|---|
| Select All Patterns | Toggle every pattern for export | |
| Select All Graphics/Top-stitches | Toggle graphics and topstitches; dropdown allows individual selection | 2.2.0 |
| Single Object | One object for the entire selection (mesh name may match sub-object name, file name, or be manual) | name options 5.1.4 |
| Multiple Objects | Each selected item as its own object | |
| Unweld | Sewn pattern vertices exported separately, as multiple objects | |
| Weld | Merge vertices of sewn pattern parts into shared points | |
| Weld Selected Pattern | Weld only along sewing lines between selected patterns | 2.3.0 |
| Weld Turned Sewing Lines | Weld even when Sewing Line Type is Turned | 2026.0 |
| Thin | No thickness in geometry even if thickness was increased | |
| Thick | Export thickness; Quad pattern mesh only | 2.3.0 |
| Combined (Unified) UV Coordinates | Fit all pattern textures to UV 0..1 (Unified Texture) | 3.1.0 |
| Include Internal Shape | Internal lines and shapes included | 2.3.0 |
| Scale | Value, or unit chosen from "Unit in file to import" | |
| Axis Conversion | Direction of each axis | |
| Save with Texture File (.zip) | Package .obj, .mtl and textures | |
| Diffuse Color Combined on Texture | Burn fabric color into the texture image | 2.2.0 / 4.2.0 |
| Exclude Ambient Map & Color (Modo) | Drop ambient map and color from materials | 2.2.0 / 4.2.0 |
| Save with Alpha Map | Include alpha map | 4.2.0 |
| Save with Meta Data (.xml) | Pattern mesh, physical props, sewing info | 2.3.0 |

Related notes: front, side and back meshes can be exported separately; internal lines can be exported also to FBX.

Sources: OBJ Import/Export page [verified x2: manual page and API enum page].

### 3.2 OBJ sequence export

All OBJ options plus:

| Option | Meaning |
|---|---|
| Image File | Save path and base name; numbers appended (_0, _1) |
| Texture Size | Width and height at fixed ratio, 100 to 10000 px |
| Fill Texture Seams | Extend texture beyond boundary 0 to 15 mm, previewed in 2D window |
| Animation: Entire Area / Play Area Only | Frame range |
| Input Frame | Specified frame range | 5.1.4 |
| Save with Bone Information (JSON) | Extra bone data per frame beside OBJ files |

### 3.3 FBX export

| Option | Meaning | Version |
|---|---|---|
| Select an Avatar / Select All Avatars | Export one or all avatars | 6.1 / 12.2 |
| Single / Multiple Objects | As for OBJ | |
| Un-Weld, Weld, Weld Selected Pattern | As for OBJ | 2.3.0 |
| Weld Turned Sewing Lines | | 2026.0 |
| Thin / Thick | Thick requires Quad mesh; in thick mode each side (front/back/sides) texture exports separately; thin single object exports a single whole texture | 2.3.0, 5.0.0 |
| Unified Texture | Listed, no detail on page | 3.1.0 |
| Joint Animation | Save animation data originally imported with the FBX | changed 2026.0 |
| Cache Animation (MC) | Save cache data with recorded garment animation | changed 2026.0 |
| Scale | Value or unit | |
| Axis Conversion | Axis direction | |
| Animation Entire Region / Play Region Only | Frame range | |
| File Type | Binary or ASCII | |
| Relative Texture Image File Path | Name only; images must sit beside the FBX | |
| Absolute Texture Image File Path | Full original path stored | |
| Save With Texture Files (.zip) | Archive with textures | |

Notes: FBX SDK 2020 available (ver 10). The API exposes `fbxSdkVersion` codes: 0=2020, 1=2019, 2=2018, 3=2016/2017, 4=2014/2015, 5=2013, 6=2012. Material names are exported with FBX synchronized (2025.0). Only FBX-imported joints can be exported as FBX [verified x2: FBX page and compatibility page].

### 3.4 FBX import

| Option | Meaning |
|---|---|
| Load Type Open / Add | Replace scene vs add to it |
| Auto Create Fitting Suit | Humanoid avatars only |
| Object Type: Avatar / Trim / Scene & Props | Avatar (multiple allowed); Trim passes through cloth (no collision unless Collision enabled in Property Editor; has Glue and Scale gizmo icons); Scene & Props with collision settings |
| Object Camera | Import camera / camera animation |
| Joint Animation | Import joint animation |
| Cache Animation (MC) | Import cache animation |
| Automatically Add Arrangement Points | Humanoid avatars only |
| Scale | Value or Measure |
| Axis Conversion | Axis direction (12.2) |
| Translation: Align Bottom to Ground | Lift objects below the floor |
| Move Garment to Start Position | Move to first frame of imported motion |

### 3.5 Alembic export (File > Export > Alembic)

Three variants listed: Alembic (ver 6.1 generic), Alembic (HDF5), Alembic (OGAWA). Ogawa is the modern default layout; HDF5 is the legacy layout.

| Group | Option | Meaning |
|---|---|---|
| File | File Type | Choose Alembic backend (HDF5 or Ogawa) |
| Object | Unweld / Weld / Weld Selected Pattern (2.3.0) | As above |
| Object | Thin / Thick | As above |
| Object | Merge Vertex by Proximity | Merge vertices across patterns by distance |
| Object | Weld Turned Sewing Lines (2026.0) | As above |
| Object | Unified UV Coordinates | Fit all textures to 0..1 |
| Object | Include Avatar Shape | Save the avatar into the file |
| FPS | FPS (below ver 12.2) | Cache frame rate |
| Basic | Scale | Unit or percentage |
| Animation | Entire Region / Play Region Only; Current Frame (2024.0) | Frame range, or a single frame |
| Color | Strain / Stress | Write selected value as vertex color |

### 3.6 Alembic import

| Option | Meaning |
|---|---|
| Load Type Open / Add | |
| Scale | Value or preset |
| Axis Conversion (2024.0) | |
| Move Garment to Start Position | First frame of motion |
| Move Avatar & Garment to Start Position | |
| API `loadObjectType` | 0 = Avatar, 1 = Morph Target (others return false) |
| API `aUnit` default 0.01 (mm), `aScale` 0 to 1, `morphingFrameCount` 1 to 99 | |
| API `boolTrace2DPatternsFromUVMap`, `boolAutoCreateFittingSuit`, `boolAddArrangementPoints` | Auto features on import |

### 3.7 USD export (File > Export > USD)

| Option | Meaning |
|---|---|
| Select All Patterns | |
| Unweld / Weld / Weld Selected Pattern / Weld Turned Sewing Lines (2026.0) | As above |
| Thin / Thick | As above |
| Unified UV Coordinates | |
| Scale | Value or unit |
| Animation Entire Region / Play Region Only | Records full or play-area garment animation |

Python `ExportUSDOption` (extra fields):

| Field | Meaning |
|---|---|
| m_upAxis | 1 = Y up, 2 = Z up |
| m_metersPerUnit | double |
| m_bAssignAvatar, m_avatarPrimPath | default `/Avatar` |
| m_bAssignGarment, m_garmentPrimPath | default `/Garment` |
| m_bAssignSceneNProp, m_sceneNPropPrimPath | default `/SceneAndProp` |
| m_bExportSimulationData | Exports per-frame sim data (needed for Chaos Cloth path) |
| m_bExportStrandData | Export strand data (fur / hair strands) |

### 3.8 USD import

Scale, Move Garment to Start Position, Move Avatar & Garment to Start Position, Transition Animation (blend from T-pose to the motion's first pose when the avatar pose differs).

### 3.9 glTF 2.0 / GLB (MD 2026.0 manual)

Import: Open Type (open or add), Object Type (avatar or trim), Arrangement Points auto-generate, Scale (meters default), Axis Conversion.

Export:

| Option | Meaning |
|---|---|
| Object selection | All Avatars, Patterns, Graphics, Trims; per-object toggles |
| Single / Multiple Objects | |
| Thick / Thin | |
| Unified UV Coordinates | Combined UV set |
| Lights | Export lights |
| Animation | Garment animation, avatar joint keyframes, or cached avatar animation; Bake Keyframes creates a key per frame |
| Animation range | Entire keyed range or play region |
| Scale, Axis Conversion | Meters default |
| Textures | glTF-Embedded (texture files included) or compressed ZIP |
| Diffuse Color Combined on Texture | Hidden when Unified UV is on (color already diffused to the UV map) |
| Exclude Ambient Map & Color (Modo) | Applied automatically, menu hidden |
| Save with Meta Data (extras) | XML metadata for patterns, physical properties, sewing |

Python API note: ImportExportOption covers "OBJ/GLTF export" and has `bEmbedded` and `bCreateMetallicRoughnessMap` fields (metallic-roughness packing for glTF). [verified x2: manual and developer page]

### 3.10 Animation cache export (MDD, PC2, MC)

Menu: File > Export > Animation. Formats: Maya Cache (.mc), Maya Cache (.mcx), Point Cache 2 (PC2), MDD Cache (Standard), MDD Cache (Maya/3ds Max). Options are the shared set: Single/Multiple, weld modes, thin/thick, combined UV, Image File, Texture Size (100 to 10000), Fill Texture Seams (0 to 15 mm), Scale, Axis Conversion, Entire/Play Area, Save with Texture Files (.zip), Save with Meta Data (.xml), Diffuse Color Combined, Exclude Ambient Color (Modo). The page does not map which option applies to which format.

### 3.11 Animation cache import

Menu: File > Import > Animation. Formats: .mc, .mcx, PC2, MDD Standard, MDD (Maya/3ds Max), Cache (Maya/3ds Max).

| Option | Meaning |
|---|---|
| Object (Select All Avatars) | Apply cache to all avatars or individually |
| FPS | Only appears for PC2 |
| Scale | |
| Axis Conversion | Y vertical by default |
| Move Avatar & Garment to Start Position | |
| Transition Animation | T-pose to first pose blend |

### 3.12 OBJ import

Menu: File > Import > OBJ (new scene) or Import (Add) > OBJ.

| Object type | Meaning |
|---|---|
| Load as Avatar | Garment-collidable avatar. "Add Arrangement Points & BVs" generates bounding volumes and arrangement points (4.2.0) |
| Load as Trim | Garments pass through (no collision) (3.0.0) |
| Load as Garment | Imports as a 3D garment; "Trace 2D Patterns from UV Map" builds patterns from UVs (4.2.0). Used for OBJ from Maya / 3ds Max |
| Load as Morph Target | Morphs original object into target; Transition Frame Count sets length. Both files need identical mesh structure. File names with "#" will not load |

Shared: Scale (manual or unit), Axis Conversion, Align Bottom to Ground (4.2.0). API has `bSizeAndPoseFromAvatar`, `bAutoTranslate`, `bCreateCamera`, etc.

### 3.13 Export Selected

`OBJ (Selected)` exports only selected items in 2D / 3D windows (selected items are pre-checked in the dialog). In the API, `ExportComponentSelection` (used when `bUseSelectComponent` is true) has switches per group: pattern, avatar, graphic, button, buttonHole, topstitch, zipper, piping, trim, puckering, and name filter lists per group (empty = all).

### 3.14 Complete `ImportExportOption` field catalog (Python API)

| Category | Fields |
|---|---|
| Content | bExportGarment, bExportAvatar, bSingleObject, bThin, bIncludeHiddenObject, bIncludeInnerShape, bIncludeAvatarShape (FBX/Alembic; keep equal to bExportAvatar), bExportLight, bExportFabric, bMetaData, bSaveInZip, bEmbedded |
| Welding | weldType: 0 FULLY_UNWELDED, 1 DEFAULT_WELDED (deprecated), 2 FULLY_WELDED, 3 SELECTED_WELDED |
| UV and texture baking | unifiedTextureSize, unifiedTextureFillSeamSize, bUseInifinteSeams, unifiedTextureBakeMargin, unifiedTextureBakeRelateive, bClothUnifiedUVCoordinates, bAvatarUnifiedUVCoordinates, bUnifiedDiffuseMap, bUnifiedNormalMap, bUnifiedMetalnessMap, bUnifiedRoughnessMap, bUnifiedOpacityMap, bUnifiedDisplacementMap, bCreateMetallicRoughnessMap, bDiffuseColorCombined, bOpacityMap, bExcludeAmbient (deprecated: bUnifiedUVCoordinates, bCreateUnifiedTexture) |
| Scale and axes | scale, axisX, axisY, axisZ (0 X, 1 Y, 2 Z), bInvertX/Y/Z |
| Import placement/animation | ImportObjectType (0 Avatar, 1 Trim, 2 Garment), bAutoTranslate, bCreateCamera, bCreateAvatarJointAnimation, bCreateAvatarCacheAnimation, bCreateAvatarBlendShapeAnimation, bCreateClothCacheAnimation, bMoveGarment, bAddArrangementPoints, bAutoCreateFittingSuit, bAdd, translationValueX/Y/Z, bTrace2DPatternsUVMap, bSizeAndPoseFromAvatar |
| Object filters | patternList, buttonList, graphicList, zipperList, bUseSelectComponent, selectComponent |
| Other | m_AuthenticationKeyForAPI (for Alvanon extended functionality) |

Unit enum (`translationUnit`): 0 mm, 1 cm, 2 inch, 3 m, 4 feet. `VideoExportOption`: width, height, fps (0=24, 1=25, 2=30, 3=60, -1 system), startFrame, endFrame.

Clone implication: the weld-mode enum (unwelded, welded, selected-welded) and the per-map unified bake flags are the concrete data model to copy.

## 4. Thickness export

- Setting: Thin vs Thick in every mesh export dialog. Thin ignores the pattern's rendering thickness; Thick bakes it into geometry.
- Constraint: Thick works only when the pattern mesh type is Quad (ver 2.3.0 note, repeated on FBX, Alembic, OBJ, cache pages). Triangle meshes do not export thickness. [verified x2]
- The "Add Pattern Thickness" page exists (article 47358238232985) but was not fetched, so thickness parameter ranges are [not verified].

## 5. Mesh type, quad mesh, remesh

### 5.1 Mesh types

| Mesh type | Where to set | Use |
|---|---|---|
| Triangular | Property Editor > Miscellaneous > Mesh Type; recommended default "in all cases except exceptional cases" | General |
| Quadrangular (Quad) | Same; right-click pattern in 3D window > Quadrangulate | Smooth rounded details such as collars and piping |
| Triangle/Quad (Optimized) | Mesh Style (2024.0); Rectangle (Optimize) algorithm improved in 2024.0 and 2025.0 | Hybrid |
| Quad (Grid) | Geometry > Mesh Style (2024.0 / 2025.0) | Aligned grid quad mesh |

Recommended workflow: set all patterns Triangular, change only needed ones to Quad. Speed varies by mesh type. [single source: Quadrangular/Triangular Mesh page; Quad (Grid) and Optimized not defined on page]

### 5.2 Auto Remeshing (aligned quad retopology)

- Access: right-click pattern in 3D window > Remeshing; or Mesh Style > Quad (Grid) (2024.0); or Pattern property window > Remeshing.
- Behavior: rearranges the pattern mesh into an aligned quad mesh. Particle distance of the pattern carries over. Rotating a pattern in 2D realigns mesh to 2D X/Y axes; to refresh direction after rotating, undo Remeshing and apply again.
- No user-tunable parameters documented besides particle distance. [single source]
- There is also a "Remeshing to Retopology (Selected) (ver 5.1.4)" article (not fetched).

### 5.3 Retopology (manual topology authoring, ver 5.1.0)

| Tool or command | Behavior |
|---|---|
| Topology tool (create) | In 2D or 3D toolbar. Click highlighted pattern to add points, one pattern at a time. Closed loop becomes a face only with 3 or 4 points. Double-click to finish open shapes. Delete / Backspace removes last point; Esc or Ctrl+Z cancels all. Shift+Ctrl over nearest line adds loop face division |
| Edit Topology tool | Shift multi-select; Ctrl-drag select in 2D; double-click a line to loop select; Delete; Shift+Ctrl drag to extrude lines |
| Delete | Remove selected |
| Merge | Combine points |
| Merge All | Merge points within an entered distance |
| Add Division (Current Pattern) | One subdivision level to selected pattern |
| Add All Division | Whole topology face (background right-click) |
| Remesh (duplicate) | New mesh from one pattern's topology faces, original kept |
| Remesh All (duplicate) | Same for all patterns |
| Remesh (replace) | Delete original pattern, replace with new mesh |
| Remesh All (replace) | Same for all |
| Show topology | 3D menu > Display > 3D garment > Show topology |
| Ghost display and snapping (above ver 11) | Topology points on a sewn edge show ghost points on the opposite sewing line; new points snap to them so topology aligns across seams |
| Patch Topology (above ver 11, 2D toolbar) | Subdivisions count; Create converts closed patch lines to faces; Clear deletes in-progress lines; triangular or rectangular patches display dashed internal subdivisions |

Not verified: any "Quad Remesh" command with target face counts or edge-flow guides (the docs found describe only the above). No LOD generation tool was found in official docs: [not verified].

## 6. UV Editor mode

Access: Window > UV Editor (2025.0), right-click viewport title bar > UV Editor (2025.0), or top toolbar button (up to ver 10).

| Feature | Detail |
|---|---|
| Coordinates | U horizontal, V vertical. One U/V square equals a tile; UV snapshot saves each tile as an image |
| Wire | UV polygon lines; Show Wire (7.5) via background right-click hides textures and shows the wire; wire is saved with the UV |
| View UV Maps (ver 10) | Switch the 2D window between Diffuse, Metalness, Normal, Roughness (ver 9 toolbar rendering options), plus ver 11 additions: garment OBJ topstitch UV map and displacement view/bake |
| Texture Repeat Type / Texture Mapping | Set fabric to Unified Map (10) / Unified (11) so moving UVs moves textures on 2D and 3D patterns; Repeat U and Repeat V (1 = repeats once across 0..1) |
| Move UV with Value (ver 11) | Move UV tool opens a window; Distance 1 = length of one UV tile segment; arrow icons or keys |
| Edit UV location | Page exists (article 47358235084313), not fetched |
| Normal map blending method | Page exists (47358274479129), not fetched |
| Bake Textures | See section 7 |

Important modeling fact: MD patterns are 2D by construction, so the pattern layout IS the UV (unified UV is a repack of pattern shapes into 0..1). MD does not expose seam-cutting, unwrapping (LSCM/ABF) or auto-pack algorithms on arbitrary 3D meshes in the pages retrieved. "UV seams" and "pack" as separate tools: [not verified]. The "Unfold" command in the manual is a 2D pattern operation (mirror a half pattern across a fiducial segment to make a full pattern, segment disappears, no options), NOT a UV unwrap.

### 6.1 UDIM / tiles

Tiles are explicit in the UV Editor and Bake Textures (see 7). UDIM naming (1001, 1002...) is [not verified] in primary docs.

## 7. Texture baking (UV Editor > Bake Textures tool)

Menu: Window > UV Editor Mode > toolbar > Bake Textures Tool.

| Setting | Meaning |
|---|---|
| Saving Path | Folder and name; PNG (transparent background) or JPEG (black background) |
| Size (pixels) | Output square resolution (always square) |
| Fill Texture Seams pixel value (2024.1) | Edge padding of UV shells, 1 to 256 px |
| Infinite Seams (2024.1) | Remove the padding limit, expansion without maximum and without overlapping other shells |
| UV area | 0-1 (default single tile), Selected tile (enter U,V), All tile (every tile containing UV data) |
| Texture type | Choose which maps to bake |

Version notes: ver 10 bakes seamline normal map with others; ver 11 adds the topstitch UV map and displacement map view/bake. Bakeable map types by API flags: diffuse, normal, metalness, roughness, opacity, displacement. Ambient occlusion baking: [not verified] (not in retrieved pages).

Fabric texture side: "Fabric Texture Normal Map" and "Set Fabric Textures" pages exist; `AutoGenerateFabric{NormalMap,OpacityMap,DisplacementMap}` exist in FABRIC_API (auto-generate maps from base image). Substance .sbsar can be baked to channels.

## 8. Unified UV summary (export-side)

- Dialog flag: "Unified UV Coordinates" (OBJ, FBX note, Alembic, USD, glTF, cache). Fits all garment pattern textures into one 0..1 UV set and combines texture layers into one texture (3.1.0).
- API controls: unifiedTextureSize, unifiedTextureFillSeamSize, bUseInifinteSeams, unifiedTextureBakeMargin, per-map bUnified* flags, separate cloth vs avatar flags (bClothUnifiedUVCoordinates, bAvatarUnifiedUVCoordinates).
- Texture size range in the animation/OBJ-sequence dialog: 100 to 10000 px; seam fill 0 to 15 mm.
- Interaction: with Unified UV on, "Diffuse Color Combined on Texture" is hidden in glTF (color already baked).

## 9. Python API and automation

Sources: developer.marvelousdesigner.com (API 0.1 docs), support Python Script page, DigitalProduction news (secondary).

| Topic | Detail |
|---|---|
| Editor | Main Menu > Plugins > Python Editor, or Window > Script Python |
| Open script | File > Import > Python Script File (.py), or drag and drop |
| Toolbar | New Document, Save, Save As, Run Script, Source, Open, Script Editor, Console Log |
| Plugin registration | `.py` files can be added as plugins (MD 2025.1.107); Plug-in Management page exists, not fetched |
| pip | Built-in pip for Windows Python added in 2025.1.107 |
| Platforms | Windows, macOS, Linux (Linux release 2025-09-30; Rocky Linux 9, RHEL 9 and equivalents) [Linux facts single source, secondary press article] |
| Python version | [not verified] |
| GUI API | Docs reference a GUI API with requirements, "for an upcoming update" |
| REST calls | Removed from docs in 2026.1 |

Modules and key functions (from API List page):

| Module | Functions (selection) |
|---|---|
| EXPORT_API | ExportZPac, ExportZPrj, ExportOBJ, ExportAlembic, ExportFBX, ExportUSD(path, options, usdOptions), ExportAVT, ExportPose, ExportTopStitchStyle, ExportZCMR, GenerateZcmrFrom3DWindow, ExportThumbnail2D/3D, ExportSnapshot2D/3D, ExportStdViewImage, ExportCustomViewSnapshot, ExportTurntableVideo, ExportTurntableImages, ExportAnimationVideo, GetAvatarCount/NameList/GenderList |
| IMPORT_API | ImportFile, ImportZprj, ImportZpac, ImportOBJ, ImportFBX, ImportAlembic, ImportSMP, ImportGraphicStyleFromImage, ImportAsGraphic, ImportTrim, ImportAvatar, ImportAvatarMeasurement, ImportPose |
| FABRIC_API | selection and info, Add/Replace/Export/Delete fabric, ExportZFab, CreateZfabFromTextures, CombineZfab, ImportSubstanceFile(+AsFaceType), SetSubstancePreset/Resolution, AssignFabricToPattern, metadata Get/SetFabricInformation, PBR base color/displacement/roughness/metalness/normal/opacity Get/Set, AutoGenerate maps, TransformAOPOnFabric, AddTextureToPatterns |
| PATTERN_API | pattern counts and info, area, line length, particle distance Get/Set, mesh type Get/Set (`SetMeshType`), shrinkage Get/Set, bounding box, arrangement Get/Set, copy/move/delete pieces and points (list truncated in source) |
| UTILITY_API | not captured (page truncated) |

Notable behaviors: export in Python still pops up the dialog in some cases (community report, not used as evidence). Changelog highlights: ImportAlembic added 2024.2.189; RunAnimationRecording, MoveAnimationFrame (2025.0.157); AlignAvatarsAndGarmentToCenter, AddGraphicToPattern, Get/Set AvatarTextureMap, SetViewControlsDefaults, SetFormat3DBackground, SetTrimDisplaySettings (2025.0/2025.1); 139 functions synced in 2026.1 including ExportPatternCPX / ImportPatternCPX; removals in 2026.1 include V-Ray functions, shirring, grading, baselines, tech-pack category, CLO plug-in widget helpers.

Clone implication: expose a headless scripting layer mirroring this module split (export, import, fabric, pattern, utility), and include batch import/export, simulation parameter control, and turntable/snapshot rendering.

## 10. Unreal Engine pipelines

### 10.1 LiveSync (MD to UE)

| Item | Detail |
|---|---|
| What | Plugin moving assets between MD and UE in one click: meshes, materials, skeletal animation, geometry caches |
| Flow | Install and activate LiveSync in UE; keep MD running; open LiveSync Editor; click Update; MD scene appears with auto-generated shader balls per fabric |
| Setting | Unified Material: enable, then Update again, generates unified materials from UV coordinates |
| Characters | Skeletal meshes including MetaHuman can be exported to and from UE |
| Use | Rendering, cinematics, Sequencer |
| Release | CLO LiveSync 2.0.0 released for CLO 2025.0 (2025-04-11); MD LiveSync 2.0.0 tied to MD 2025.0; a notice exists about an issue with CLO/MD LiveSync 2.2.2 (not read) |
| Limits | MD must be running; supported version matrix not listed on fetched pages |

[single source for settings; announcement confirms existence of LiveSync 2.0.0]

### 10.2 Chaos Cloth / MetaHuman via USD

Compatibility (official MetaHuman workflow article): MD 2025.0 works only with UE 5.4; MD 2025.2 works with UE 5.6 and later; UE 5.5 is incompatible with ChaosCloth from this pipeline (simulation data issues). Restrictions apply only to ChaosCloth; regular meshes and materials unaffected.

Workflow:

1. Export garment from MD as USD with simulation data (ExportUSDOption `m_bExportSimulationData`).
2. Import USD into UE content drawer, confirm static meshes.
3. Prepare MetaHuman: export Combined Skeletal Mesh from the MetaHuman Editor.
4. Create Cloth Asset (Physics > Cloth Asset), open Dataflow Editor, keep nodes: USD Import (Static Mesh Import), Transfer Skin Weights, Remesh, optional Remesh LOD2, Cloth Asset Terminal.
5. Transfer Skin Weights: Target Meshes = Render Mesh, Render Mesh Transfer = Skeletal Mesh, Transform Method = Closest Point on Surface, Skeletal Mesh = Combined Skeletal Mesh.
6. Create Outfit Asset (Resizable Outfit Template), set Evaluate Dataflow Graph Manual, add Sized Outfit Source, assign cloth asset, evaluate, add source body slider, enable Sized Outfit, evaluate, switch back to Automatic.
7. Apply in MetaHuman Editor Hair and Clothing tab > Outfit Clothing.

The article does not specify MD export settings beyond USD. UE's own Cloth/Dataflow documentation at dev.epicgames.com was not fetched (budget), so UE-side options are [single source].

## 11. DCC and other pipelines (what is officially documented)

| Target | Documented path | Evidence |
|---|---|---|
| Unreal Engine | LiveSync plugin; USD + Chaos Cloth; FBX/Alembic | Verified above |
| Maya / 3ds Max | OBJ garment import (OBJ to Garment, Maya or 3ds Max source); MC/MDD/PC2 point caches; FBX; Collada import (OpenCOLLADA plugin); MDD (Maya/3ds Max) variants | Compatibility and cache pages |
| Modo | LXO export; OBJ "Exclude Ambient (Modo)" options | Compatibility page |
| Substance | .sbsar import with channel bake; Substance functions in FABRIC_API | Compatibility page and API |
| ZBrush (GoZ), Blender, Houdini, Cinema 4D | No dedicated official plugin found; generic OBJ/FBX/Alembic/USD/glTF round trip. GoZ: [not verified] | Not found |
| Daz, Character Creator, iClone, Unity | No official plugin found in retrieved pages. Avatar import accepts OBJ/FBX/Collada; Character Creator and iClone interchange is [not verified] | Not found |
| Alvanon | Auth key field in ImportExportOption (`m_AuthenticationKeyForAPI`) for extended functionality | Developer page |
| CLO (Virtual Fashion) | Same ZPRJ/ZPAC formats, shared company; exact cross-open compatibility rules [not verified]. CLO LiveSync is the sibling product of MD LiveSync | LiveSync news |

## 12. Import/export presets

An "Import and Export Preset" article exists (48920133557145) but could not be fetched, so preset contents are [not verified]. A clone should still support named saved option sets since every dialog has 10+ fields.

## 13. CONNECT, Library and MD Cloud

| Item | Detail |
|---|---|
| CONNECT Store | Store tab in left panel; as of 2025.0 integrated into the New Library Window (Beta). Item list with hover info, likes, cart, keyword search, category filter, sort; "Don't show owned item" for signed-in users; item detail with seller, tags, description, images, 3D viewer button; Licenses Basic or Extended; cart and checkout (opens web purchase page); My Item list with Download button; sign-in with MD website account |
| 2026.0 CONNECT | New setting to choose the download folder path; "Preview in docked library" |
| MD Cloud | [not verified]: no official page named MD Cloud found. Do not assume a feature. |
| Uploading/selling | Store page does not describe upload flow |

## 14. MD 2026.0 feature list relevant to I/O

The 2026.0 feature list mentions no import/export items other than the CONNECT download folder, while the Ver 2026.0 manual pages add "Weld Turned Sewing Lines" and changes to FBX Joint Animation / Cache Animation save behavior, plus glTF 2.0 import/export pages. Other 2026.0 features: Lacing tool, 3D pencil, preserve 3D shape when scaling 2D pattern, EveryWear template rigging (beta), toon shader, GPU trim simulation, pattern archive, fabric-aware strain maps.

## 15. Build checklist for formfabric (derived from the above)

1. One `ExportOptions` struct: content flags, weldType (3 values), thin/thick, singleObject, unified UV block (size, padding, infinite seams, per-map flags), scale + unit, axis map + invert flags, animation range (entire, play, current frame, explicit range), metadata, zip.
2. Writers: OBJ+MTL, OBJ sequence, FBX (binary/ascii, relative/absolute textures), Alembic (Ogawa mandatory, HDF5 optional, strain/stress vertex colors), USD (upAxis, metersPerUnit, prim paths, sim data, strands), glTF/GLB (embedded or zip, KHR lights, baked keyframes, metallic-roughness), MDD/PC2/MC point caches.
3. Readers: OBJ (avatar/trim/garment/morph), FBX, Alembic, USD, glTF, Collada, caches, SBSAR, ZPRJ/ZPAC.
4. Mesh types: triangle, quad, optimized, quad grid; thick export only when quad.
5. Topology authoring (3/4-point faces, merge, division, remesh duplicate/replace, ghost snapping across seams, patch tool) plus auto quad grid remesh aligned to the 2D pattern axes.
6. UV: unified 0..1 packing, tile selection, bake margin, per-map bake.
7. Scripting API mirroring module split; headless on Linux.
8. Live link: socket plugin for UE (shader balls per fabric, unified material toggle); USD with per-frame simulation data for Chaos Cloth.

## Not verified list

Collada export, PLY, DXF/AI import details, GoZ, Blender/Houdini/Cinema 4D/Daz/Character Creator/iClone/Unity official bridges, MD Cloud, UDIM naming, AO bake, LOD generation, auto seams/unwrap, Python version, preset contents, thickness parameter ranges, LiveSync supported version matrix.

## Sources (URLs fetched)

- https://support.marvelousdesigner.com/hc/en-us/articles/47358223288601 (OBJ Import/Export)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358253053081 (Alembic Import/Export)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358305803673 (USD Import/Export)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358232885017 (FBX Import/Export)
- https://support.marvelousdesigner.com/hc/en-us/articles/55686813557913/ (glTF, Ver 2026.0)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358272195609 (Compatible File Format)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358169252633-Marvelous-Designer-File-Format
- https://support.marvelousdesigner.com/hc/en-us/articles/47358266324377 (ZPrj/zpac metadata)
- https://support.marvelousdesigner.com/hc/en-us/articles/47358355178393-Cache-Animation-Export
- https://support.marvelousdesigner.com/hc/en-us/articles/47358364600601-Animation-Import
- https://support.marvelousdesigner.com/hc/en-us/articles/47358384957209-OBJ-Sequence-Export
- https://support.marvelousdesigner.com/hc/en-us/articles/47358367597081-Quadrangular-Triangular-Mesh
- https://support.marvelousdesigner.com/hc/en-us/articles/47358347315481-Remeshing
- https://support.marvelousdesigner.com/hc/en-us/articles/47358219101081-Retopology-ver-5-1-0
- https://support.marvelousdesigner.com/hc/en-us/articles/47358434063001-Unfold
- https://support.marvelousdesigner.com/hc/en-us/articles/47358198817689--Mode-UV-EDITOR
- https://support.marvelousdesigner.com/hc/en-us/articles/47358222133657--MODE-UV-EDITOR-Bake-Textures
- https://support.marvelousdesigner.com/hc/en-us/articles/47358145573401 (Tips: MD and Unreal Engine, LiveSync)
- https://support.marvelousdesigner.com/hc/en-us/articles/52699135975705-Marvelous-Designer-to-MetaHuman-USD-Garment-Integration-Workflow
- https://support.marvelousdesigner.com/hc/en-us/articles/55837641308313-Marvelous-Designer-2026-0-New-Feature-List
- https://www.marvelousdesigner.com/manuals/articles/47358320627865 (Connect Store)
- https://www.marvelousdesigner.com/support/news/view/ab037770b24049bd9426486574a4e1fc (LiveSync 2 announcement)
- https://developer.marvelousdesigner.com/ (API home)
- https://developer.marvelousdesigner.com/python.html
- https://developer.marvelousdesigner.com/optiontype.html
- https://developer.marvelousdesigner.com/list.html
- https://developer.marvelousdesigner.com/changelog.html
- https://digitalproduction.com/2025/09/18/marvelous-designer-finally-sews-itself-into-linux/ (trade press, secondary; used only for Linux/Python facts)

Fetch failures (not used as evidence): manual.marvelousdesigner.com Quadrangulate (robots disallowed), Import and Export Preset article, Unified Texture article, article 47358358378905.
