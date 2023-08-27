pub fn as_const_ptr<Struct, Type>(data: &Struct) -> *const Type {
    data as *const Struct as *const Type
}

pub fn as_mut_ptr<Struct, Type>(data: &mut Struct) -> *mut Type {
    data as *mut Struct as *mut Type
}

pub fn as_mut_slice<'a, Type, OutType>(data: &mut Type, len: usize) -> &'a mut [OutType] {
    unsafe {
        let mut_ptr = as_mut_ptr::<Type, OutType>(data);
        return std::slice::from_raw_parts_mut(mut_ptr, len);
    }
}
