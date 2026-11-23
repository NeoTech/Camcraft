## #3450 - DXF import skips the CLASSES, THUMBNAILIMAGE and ACDSDATA sections

**Chosen.** The DXF reader skips three inert sections that the issue names: `CLASSES`, `THUMBNAILIMAGE` and `ACDSDATA`. It reads their records to the matching `ENDSEC` and imports the rest of the file. The content of these sections is opaque. Group pairs right after the section name are allowed, because `THUMBNAILIMAGE` and `ACDSDATA` hold data there. A section with no `ENDSEC` (end of file or a new `SECTION` first) gives the named error `<SECTION>: missing ENDSEC`. Every other unknown section name stays refused with `unsupported SECTION`.

**CLASSES strictness.** `CLASSES` is opaque too. A strict check for `CLASS` records only could refuse a legal file, and no test of ours proves that every legal `CLASSES` section holds only `CLASS` records. The test `classes_section_holds_only_class_records` is dropped.

**Why.** These sections hold class definitions, a preview image and application data. They hold no geometry. Refusing them rejected files that hold valid entities. Reading to `ENDSEC` cannot hang or panic.

**Dropped.** Skip every unknown section: this would hide a real unsupported file. Parse the section content: the importer has no use for it. A strict `CLASS` record check: see above.

**Cost to change.** Low. The change is one `matches!` arm and one function in `crates/geometry/src/dxf/reader.rs`.

### Part 2 - the OBJECTS section of an AC1015 file

**Second cause.** With the sections skipped, the real AC1015 exports still failed in `OBJECTS`. The reader refused `DICTIONARY: unsupported dictionary group 102 or value`, because each object carries an application-defined reactor bracket (`102 {NAME` .. `102 }`). Behind that refusal it also refused every object type other than `DICTIONARY` (the exports have `LAYOUT`, `ACDBDICTIONARYWDFLT` and `ACDBPLACEHOLDER`).

**Chosen.**
- A group 102 bracket is opaque, in a `DICTIONARY` and in the header of any other object. It must open with `{` and a name, close with `}`, never nest, and close before its record ends. Each other form is a named error (`unclosed group 102 block`, `group 102 closes no open block`, `nested group 102 block`, `malformed group 102 block marker`).
- A `DICTIONARY` keeps all its field checks from #3176. Only the bracket content is now outside those checks, so a reactor `330` is not a second owner.
- An object of any other type is an inert record. The reader reads only its common header (handle 5, brackets, owner 330, up to the first subclass marker 100) and does not interpret the rest. The object needs a non-zero, unique handle and a non-zero owner. It joins the ownership graph of #3176: its owner must resolve to an object in the section, a dictionary entry that names it must name its true owner, and the chain must end at a root.
- A record with the `AcDbEntity` subclass in `OBJECTS` is refused (`graphical entity in OBJECTS`). A file that ends inside `OBJECTS`, or meets `EOF` or `SECTION` there, gives `OBJECTS: missing ENDSEC`.

**Why.** The reader keeps no `OBJECTS` data: `dictionaries::read` returns only pass or error, and nothing downstream reads dictionaries or objects. Objects are not drawn. A drawn item is always an entity in `ENTITIES` or `BLOCKS`, and those readers stay strict: unknown entities, paper space, external references and group 102 in an entity (which an extension dictionary or a clip filter needs) stay refused. So no object can add, remove or move imported geometry, and no object can change the units (`$INSUNITS` is in `HEADER`). The checks that stay are structural: the ownership graph, the bracket balance and the entity marker.

**How this sits with #3176.** The #3176 decision dropped "skip the section as a whole", because that could hide drawing objects, and it dropped acceptance by familiar dictionary names. This change does neither. `OBJECTS` is still read record by record. No object type and no entry name is accepted by its name. An unowned record, a stray entity and a broken bracket are still refused. The #3176 cost line ("another object type requires its own field checks and tests for its effect on imported geometry") is met by one general rule: an inert object has no effect on imported geometry because the reader takes no value from it.

**Pins.** No existing pin changed. `dictionary_entries_must_form_a_closed_acyclic_ownership_graph` keeps its `XRECORD` case (now refused as `missing object owner`) and its `LINE` case (now refused as `graphical entity in OBJECTS`, see Part 3). The group 340 and foreign subclass cases in a `DICTIONARY` stay refused.

**Dropped.**
- Skip `OBJECTS` as a whole, as for `CLASSES`: #3176 rejects this.
- Accept only `LAYOUT`, `ACDBDICTIONARYWDFLT` and `ACDBPLACEHOLDER` by name: other exporters write other object types (`XRECORD`, `GROUP`, `MLINESTYLE`) with the same effect on geometry, which is none.
- Read `ACDBDICTIONARYWDFLT` as a dictionary: its default pointer (340) and second subclass give the reader nothing to use.
- Check group 102 also after the first subclass marker: that part of an object is opaque data, and a check there could refuse a legal file without a gain.

### Part 3 - second review: no drawable record in a skipped place

**Gap.** A review of Part 1 and Part 2 found one way to lose geometry silently. A record with an entity name (`LINE`, `LWPOLYLINE`, `INSERT` and so on) but without the `AcDbEntity` subclass marker passed as an inert object in `OBJECTS`. A record of that kind in `CLASSES`, `THUMBNAILIMAGE` or `ACDSDATA` was skipped. A legal file never puts an entity there, but a damaged or hand-edited file can. The import then had fewer outlines than the file shows, with no diagnostic.

**Chosen.** The reader refuses a record whose name is in the entity list of the DXF Reference, in each of these places. The list is `ENTITY_KINDS` in `crates/geometry/src/dxf/entities.rs`, compared without case. The errors are `<KIND>: graphical entity in OBJECTS` and `<SECTION>: <KIND> entity in a section without geometry`. This is a refusal by name only. No record is accepted by its name, so the #3176 rule against acceptance by familiar names still holds.

**Dropped.** Require only `CLASS` records in `CLASSES`: see "CLASSES strictness" above. A list of known object names: it would refuse object types of other exporters, with no gain for the geometry.

### How the #3176 law changed

The #3176 decision protects one thing: no block geometry, external reference or drawing object can hide in a section that the reader does not read. It does this with three rules. (1) A section is never skipped as a whole when it can hold such data. (2) A record is never accepted only because its name is familiar. (3) Each object type that the reader accepts has its own checks and tests for its effect on the imported geometry.

After this change:
- Rule 1 is unchanged for `BLOCKS`, `OBJECTS`, `TABLES` and `ENTITIES`. Only `CLASSES`, `THUMBNAILIMAGE` and `ACDSDATA` are skipped. The DXF Reference gives them class definitions, a preview bitmap and application data only. An entity record in them is refused (Part 3).
- Rule 2 is unchanged. No object type and no dictionary entry name is accepted by its name.
- Rule 3 is replaced for objects other than `DICTIONARY` by one general rule: an object that is not a `DICTIONARY` is inert. The reader takes no value from it. It must have a handle, an owner and balanced group 102 brackets. It must join the ownership graph and must not carry the `AcDbEntity` marker or an entity name. An object that is drawn needs an entity in `ENTITIES` or `BLOCKS` (for example `IMAGE` for an `IMAGEDEF`), and those readers still refuse each unsupported entity.
- `DICTIONARY` keeps all its field checks. The only change is that a group 102 bracket is now read as opaque data.

**Real files.** The three QA files of the issue were read through `dxf::parse` and `SketchData::from_dxf` on the final branch. They are not committed, because they are not ours to commit. `q04-vcarve-sign-letters.dxf` gives 300 primitives and 2 closed chains (area 261.70 mm² each). Its R12 twin gives 894 primitives and 2 closed chains (261.79 mm² each: the R12 export has line segments for the arcs). `q06-die-lines.dxf` gives 117 primitives and 1 closed chain. The hand-written fixture `crates/geometry/tests/dxf_ac1015_fixture.rs` has the same section and object layout and pins this.

**Not covered.** An extension dictionary owned by an entity or a table record still fails the owner check (`unresolved dictionary owner`), as before this change.

**Cost to change.** Low. The rule is one function and one bracket type in `crates/geometry/src/dxf/dictionaries.rs`, pinned by `crates/geometry/tests/dxf_objects_section.rs`.
