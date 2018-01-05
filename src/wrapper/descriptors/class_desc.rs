use strings::JNIString;

use objects::{
    AutoLocal,
    AttachedGlobalRef,
    GlobalRef,
    JClass,
    JObject,
};

use descriptors::Desc;

use JNIEnv;

use errors::*;

impl<'a, T> Desc<'a, JClass<'a>> for T
where
    T: Into<JNIString>,
{
    fn lookup(self, env: &JNIEnv<'a>) -> Result<JClass<'a>> {
        env.find_class(self)
    }
}

impl<'a, 'b> Desc<'a, JClass<'a>> for JObject<'b> {
    fn lookup(self, env: &JNIEnv<'a>) -> Result<JClass<'a>> {
        env.get_object_class(self)
    }
}

/// This conversion assumes that the `AttachedGlobalRef` is a pointer to a class object.
impl<'a, 'b> Desc<'a, JClass<'b>> for &'b AttachedGlobalRef {
    fn lookup(self, _: &JNIEnv<'a>) -> Result<JClass<'b>> {
        Ok(self.as_obj().into())
    }
}

/// This conversion assumes that the `GlobalRef` is a pointer to a class object.
impl<'a, 'b> Desc<'a, JClass<'b>> for &'b GlobalRef {
    fn lookup(self, _: &JNIEnv<'a>) -> Result<JClass<'b>> {
        Ok(self.as_obj().into())
    }
}

/// This conversion assumes that the `AutoLocal` is a pointer to a class object.
impl<'a, 'b, 'c> Desc<'a, JClass<'b>> for &'b AutoLocal<'c>
where
    'c: 'b,
{
    fn lookup(self, _: &JNIEnv<'a>) -> Result<JClass<'b>> {
        Ok(self.as_obj().into())
    }
}
