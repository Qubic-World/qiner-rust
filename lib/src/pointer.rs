pub fn as_const_ptr<Struct, Type>(data: &Struct) -> *const Type {
    data as *const Struct as *const Type
}
pub fn as_mut_ptr<Struct, Type>(data: &mut Struct) -> *mut Type {
    data as *mut Struct as *mut Type
}
