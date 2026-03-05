fn main() {
    let size = size_of<u32>();
    let align = align_of<bool>();
    let offset = offset_of<std::thread::Thread>(id);
}
