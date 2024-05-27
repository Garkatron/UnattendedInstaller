extern crate winres;

fn admin(){
    let mut res = winres::WindowsResource::new();
    res.set_manifest(r#"
        <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
            <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
                <security>
                    <requestedPrivileges>
                        <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
                    </requestedPrivileges>
                </security>
            </trustInfo>
        </assembly>
    "#);
    res.compile().unwrap();
}

fn main() {
    static_vcruntime::metabuild();

    slint_build::compile("ui/appwindow.slint").unwrap();
    admin()
}
