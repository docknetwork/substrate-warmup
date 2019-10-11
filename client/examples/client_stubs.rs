use client::method::StateGetMetadata;
use client::testnode::RunningFullNode;
use core::any::Any;
use srml_metadata::{
    DecodeDifferent, FunctionMetadata, ModuleMetadata, RuntimeMetadata, RuntimeMetadataV7,
};

fn modules_names<'a>(metadata: &'a RuntimeMetadata) -> impl Iterator<Item = &str> + 'a {
    list_modules(metadata).map(
        |ModuleMetadata {
             name,
             storage: _,
             calls: _,
             event: _,
             constants: _,
         }| decoded(&name),
    )
}

fn list_modules<'a>(metadata: &'a RuntimeMetadata) -> impl Iterator<Item = &ModuleMetadata> + 'a {
    match &metadata {
        RuntimeMetadata::V7(RuntimeMetadataV7 { modules }) => decoded(modules).into_iter(),
        _ => panic!("I only know metatdata v7."),
    }
}

fn asdf(metadata: &RuntimeMetadata) {
    for ModuleMetadata {
        name: _,
        storage: _,
        calls,
        event: _,
        constants: _,
    } in list_modules(metadata)
    {
        let calls = calls.as_ref().map(assert_decode_ref).unwrap_or(&[][..]);
        for FunctionMetadata {
            name: _,
            arguments: _,
            documentation: _,
        } in calls
        {
            // FunctionArgumentMetadata {
            //     name: "_remark",
            //     ty: "Vec<u8>",
            // },
        }
    }
}

/// get a reference to the inner value of dd
fn decoded<'a, R: ?Sized, A: AsRef<R>, B: AsRef<R>>(dd: &'a DecodeDifferent<A, B>) -> &'a R {
    match dd {
        DecodeDifferent::Decoded(m) => m.as_ref(),
        DecodeDifferent::Encode(m) => m.as_ref(),
    }
}

/// get value from Decoded variant of a DecodeDifferent
/// panic if dd is Encode variant
fn assert_decode<O, D>(dd: &DecodeDifferent<O, D>) -> &D {
    match &dd {
        DecodeDifferent::Encode(_) => panic!("dd is Encode variant"),
        DecodeDifferent::Decoded(d) => d,
    }
}

/// get value from Decoded variant of a DecodeDifferent
/// panic if dd is Encode variant
fn assert_decode_ref<R: ?Sized>(dd: &DecodeDifferent<impl Any, impl AsRef<R>>) -> &R {
    assert_decode(dd).as_ref()
}

fn main() {
    let metadata = RunningFullNode::new()
        .remote_call::<StateGetMetadata>([])
        .unwrap()
        .0
         .1;
    for name in modules_names(&metadata) {
        dbg!(name);
    }
    asdf(&metadata);
}
