#![cfg(feature = "invocation")]
extern crate error_chain;
extern crate jni;

use std::sync::{Arc, Barrier, Once, ONCE_INIT};
use std::thread::spawn;

use error_chain::ChainedError;
use jni::{InitArgsBuilder, JNIEnv, JNIVersion, JavaVM};
use jni::errors::Result;
use jni::objects::AutoLocal;
use jni::objects::JValue;


pub fn jvm() -> &'static Arc<JavaVM> {
    static mut JVM: Option<Arc<JavaVM>> = None;
    static INIT: Once = ONCE_INIT;


    INIT.call_once(|| {
        let jvm_args = InitArgsBuilder::new()
            .version(JNIVersion::V8)
            .option("-Xcheck:jni")
            .option("-Xdebug")
            .build()
            .unwrap_or_else(|e| {
                panic!(format!("{}", e.display_chain().to_string()));
            });

        let jvm = JavaVM::new(jvm_args).unwrap_or_else(|e| {
            panic!(format!("{}", e.display_chain().to_string()));
        });

        unsafe {
            JVM = Some(Arc::new(jvm));
        }
    });

    unsafe { JVM.as_ref().unwrap() }
}

fn print_exception(env: &JNIEnv) {
    let exception_occurred = env.exception_check()
        .unwrap_or_else(|e| panic!(format!("{:?}", e)));
    if exception_occurred {
        env.exception_describe()
            .unwrap_or_else(|e| panic!(format!("{:?}", e)));
    }
}

fn unwrap<T>(env: &JNIEnv, res: Result<T>) -> T {
    res.unwrap_or_else(|e| {
        print_exception(&env);
        panic!(format!("{}", e.display_chain().to_string()));
    })
}

#[test]
pub fn global_ref_works_in_other_threads() {
    let env = jvm().attach_current_thread().unwrap();

    let atomic_integer = {
        let local_ref = AutoLocal::new(&env, unwrap(&env, env.new_object(
            "java/util/concurrent/atomic/AtomicInteger",
            "(I)V",
            &[JValue::from(0)]
        )));
        unwrap(&env, env.new_global_ref(local_ref.as_obj()))
    };

    let barrier = Arc::new(Barrier::new(2));

    let jh1 = {
        let barrier = barrier.clone();
        let mut atomic_integer = atomic_integer.clone();
        spawn(move || {
            let env = jvm().attach_current_thread().unwrap();
            barrier.wait();
            for _ in 0..10000 {
                unwrap(&env, unwrap(&env, env.call_method(
                    atomic_integer.as_obj(), "addAndGet", "(I)I", &[JValue::from(-1)])).i());
            }
        })
    };
    let jh2 = {
        let mut atomic_integer = atomic_integer.clone();
        let barrier = barrier.clone();
        spawn(move || {
            let env = jvm().attach_current_thread().unwrap();
            barrier.wait();
            for _ in 0..10000 + 1 {
                unwrap(&env, unwrap(&env, env.call_method(
                    atomic_integer.as_obj(), "addAndGet", "(I)I", &[JValue::from(1)])).i());
            }
        })
    };
    jh1.join().unwrap();
    jh2.join().unwrap();
    assert_eq!(1, unwrap(&env, unwrap(&env, env.call_method(
        atomic_integer.as_obj(), "get", "()I", &[])).i()));
}


#[test]
pub fn attached_detached_global_refs_works() {
    let env = jvm().attach_current_thread().unwrap();

    let local_ref = AutoLocal::new(&env, unwrap(&env, env.new_object(
        "java/util/concurrent/atomic/AtomicInteger",
        "(I)V",
        &[JValue::from(0)]
    )));

    let global_ref_1 = unwrap(&env, env.new_global_ref_attached(local_ref.as_obj()));

    {
        let global_ref_2 = unwrap(&env, env.new_global_ref_attached(local_ref.as_obj()));
        assert_eq!(1, unwrap(&env, unwrap(&env, env.call_method(
            global_ref_2.as_obj(), "addAndGet", "(I)I", &[JValue::from(1)])).i()));
        let global_ref_2 = unwrap(&env, global_ref_2.detach());
        let global_ref_2 = global_ref_2.attach(&env);
        assert_eq!(2, unwrap(&env, unwrap(&env, env.call_method(
            global_ref_2.as_obj(), "addAndGet", "(I)I", &[JValue::from(1)])).i()));
        unwrap(&env, global_ref_2.detach());
    }

    assert_eq!(3, unwrap(&env, unwrap(&env, env.call_method(
        global_ref_1.as_obj(), "addAndGet", "(I)I", &[JValue::from(1)])).i()));
}
