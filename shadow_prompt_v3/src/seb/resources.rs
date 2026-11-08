// Patched SEB DLLs embedded at compile time, indexed by version and architecture.

pub struct PatchFile {
    pub name: &'static str,
    pub x64: &'static [u8],
    pub x86: &'static [u8],
}

pub struct SebVersion {
    pub version: &'static str,
    pub files: &'static [PatchFile],
}

pub static VERSIONS: &[SebVersion] = &[
    SebVersion {
        version: "3.8.0.742",
        files: &[
            PatchFile { name: "SafeExamBrowser.exe", x64: include_bytes!("../../seb_resources/3.8.0.742/x64/SafeExamBrowser.exe"), x86: include_bytes!("../../seb_resources/3.8.0.742/x86/SafeExamBrowser.exe") },
            PatchFile { name: "SafeExamBrowser.Client.exe", x64: include_bytes!("../../seb_resources/3.8.0.742/x64/SafeExamBrowser.Client.exe"), x86: include_bytes!("../../seb_resources/3.8.0.742/x86/SafeExamBrowser.Client.exe") },
            PatchFile { name: "SafeExamBrowser.Configuration.dll", x64: include_bytes!("../../seb_resources/3.8.0.742/x64/SafeExamBrowser.Configuration.dll"), x86: include_bytes!("../../seb_resources/3.8.0.742/x86/SafeExamBrowser.Configuration.dll") },
            PatchFile { name: "SafeExamBrowser.Monitoring.dll", x64: include_bytes!("../../seb_resources/3.8.0.742/x64/SafeExamBrowser.Monitoring.dll"), x86: include_bytes!("../../seb_resources/3.8.0.742/x86/SafeExamBrowser.Monitoring.dll") },
            PatchFile { name: "SafeExamBrowser.UserInterface.Desktop.dll", x64: include_bytes!("../../seb_resources/3.8.0.742/x64/SafeExamBrowser.UserInterface.Desktop.dll"), x86: include_bytes!("../../seb_resources/3.8.0.742/x86/SafeExamBrowser.UserInterface.Desktop.dll") },
            PatchFile { name: "SafeExamBrowser.UserInterface.Shared.dll", x64: include_bytes!("../../seb_resources/3.8.0.742/x64/SafeExamBrowser.UserInterface.Shared.dll"), x86: include_bytes!("../../seb_resources/3.8.0.742/x86/SafeExamBrowser.UserInterface.Shared.dll") },
            PatchFile { name: "SafeExamBrowser.WindowsApi.dll", x64: include_bytes!("../../seb_resources/3.8.0.742/x64/SafeExamBrowser.WindowsApi.dll"), x86: include_bytes!("../../seb_resources/3.8.0.742/x86/SafeExamBrowser.WindowsApi.dll") },
        ],
    },
    SebVersion {
        version: "3.9.0.787",
        files: &[
            PatchFile { name: "SafeExamBrowser.exe", x64: include_bytes!("../../seb_resources/3.9.0.787/x64/SafeExamBrowser.exe"), x86: include_bytes!("../../seb_resources/3.9.0.787/x86/SafeExamBrowser.exe") },
            PatchFile { name: "SafeExamBrowser.Client.exe", x64: include_bytes!("../../seb_resources/3.9.0.787/x64/SafeExamBrowser.Client.exe"), x86: include_bytes!("../../seb_resources/3.9.0.787/x86/SafeExamBrowser.Client.exe") },
            PatchFile { name: "SafeExamBrowser.Configuration.dll", x64: include_bytes!("../../seb_resources/3.9.0.787/x64/SafeExamBrowser.Configuration.dll"), x86: include_bytes!("../../seb_resources/3.9.0.787/x86/SafeExamBrowser.Configuration.dll") },
            PatchFile { name: "SafeExamBrowser.Monitoring.dll", x64: include_bytes!("../../seb_resources/3.9.0.787/x64/SafeExamBrowser.Monitoring.dll"), x86: include_bytes!("../../seb_resources/3.9.0.787/x86/SafeExamBrowser.Monitoring.dll") },
            PatchFile { name: "SafeExamBrowser.UserInterface.Desktop.dll", x64: include_bytes!("../../seb_resources/3.9.0.787/x64/SafeExamBrowser.UserInterface.Desktop.dll"), x86: include_bytes!("../../seb_resources/3.9.0.787/x86/SafeExamBrowser.UserInterface.Desktop.dll") },
            PatchFile { name: "SafeExamBrowser.UserInterface.Mobile.dll", x64: include_bytes!("../../seb_resources/3.9.0.787/x64/SafeExamBrowser.UserInterface.Mobile.dll"), x86: include_bytes!("../../seb_resources/3.9.0.787/x86/SafeExamBrowser.UserInterface.Mobile.dll") },
        ],
    },
    SebVersion {
        version: "3.10.0.826",
        files: &[
            PatchFile { name: "SafeExamBrowser.exe", x64: include_bytes!("../../seb_resources/3.10.0.826/x64/SafeExamBrowser.exe"), x86: include_bytes!("../../seb_resources/3.10.0.826/x86/SafeExamBrowser.exe") },
            PatchFile { name: "SafeExamBrowser.Client.exe", x64: include_bytes!("../../seb_resources/3.10.0.826/x64/SafeExamBrowser.Client.exe"), x86: include_bytes!("../../seb_resources/3.10.0.826/x86/SafeExamBrowser.Client.exe") },
            PatchFile { name: "SafeExamBrowser.Configuration.dll", x64: include_bytes!("../../seb_resources/3.10.0.826/x64/SafeExamBrowser.Configuration.dll"), x86: include_bytes!("../../seb_resources/3.10.0.826/x86/SafeExamBrowser.Configuration.dll") },
            PatchFile { name: "SafeExamBrowser.Monitoring.dll", x64: include_bytes!("../../seb_resources/3.10.0.826/x64/SafeExamBrowser.Monitoring.dll"), x86: include_bytes!("../../seb_resources/3.10.0.826/x86/SafeExamBrowser.Monitoring.dll") },
            PatchFile { name: "SafeExamBrowser.UserInterface.Desktop.dll", x64: include_bytes!("../../seb_resources/3.10.0.826/x64/SafeExamBrowser.UserInterface.Desktop.dll"), x86: include_bytes!("../../seb_resources/3.10.0.826/x86/SafeExamBrowser.UserInterface.Desktop.dll") },
            PatchFile { name: "SafeExamBrowser.UserInterface.Mobile.dll", x64: include_bytes!("../../seb_resources/3.10.0.826/x64/SafeExamBrowser.UserInterface.Mobile.dll"), x86: include_bytes!("../../seb_resources/3.10.0.826/x86/SafeExamBrowser.UserInterface.Mobile.dll") },
        ],
    },
    SebVersion {
        version: "3.10.1.864",
        files: &[
            PatchFile { name: "SafeExamBrowser.exe", x64: include_bytes!("../../seb_resources/3.10.1.864/x64/SafeExamBrowser.exe"), x86: include_bytes!("../../seb_resources/3.10.1.864/x86/SafeExamBrowser.exe") },
            PatchFile { name: "SafeExamBrowser.Client.exe", x64: include_bytes!("../../seb_resources/3.10.1.864/x64/SafeExamBrowser.Client.exe"), x86: include_bytes!("../../seb_resources/3.10.1.864/x86/SafeExamBrowser.Client.exe") },
            PatchFile { name: "SafeExamBrowser.Configuration.dll", x64: include_bytes!("../../seb_resources/3.10.1.864/x64/SafeExamBrowser.Configuration.dll"), x86: include_bytes!("../../seb_resources/3.10.1.864/x86/SafeExamBrowser.Configuration.dll") },
            PatchFile { name: "SafeExamBrowser.Monitoring.dll", x64: include_bytes!("../../seb_resources/3.10.1.864/x64/SafeExamBrowser.Monitoring.dll"), x86: include_bytes!("../../seb_resources/3.10.1.864/x86/SafeExamBrowser.Monitoring.dll") },
            PatchFile { name: "SafeExamBrowser.UserInterface.Desktop.dll", x64: include_bytes!("../../seb_resources/3.10.1.864/x64/SafeExamBrowser.UserInterface.Desktop.dll"), x86: include_bytes!("../../seb_resources/3.10.1.864/x86/SafeExamBrowser.UserInterface.Desktop.dll") },
            PatchFile { name: "SafeExamBrowser.UserInterface.Mobile.dll", x64: include_bytes!("../../seb_resources/3.10.1.864/x64/SafeExamBrowser.UserInterface.Mobile.dll"), x86: include_bytes!("../../seb_resources/3.10.1.864/x86/SafeExamBrowser.UserInterface.Mobile.dll") },
        ],
    },
    SebVersion {
        version: "3.10.2.920",
        files: &[
            PatchFile { name: "SafeExamBrowser.exe", x64: include_bytes!("../../seb_resources/3.10.2.920/x64/SafeExamBrowser.exe"), x86: include_bytes!("../../seb_resources/3.10.2.920/x86/SafeExamBrowser.exe") },
            PatchFile { name: "SafeExamBrowser.Client.exe", x64: include_bytes!("../../seb_resources/3.10.2.920/x64/SafeExamBrowser.Client.exe"), x86: include_bytes!("../../seb_resources/3.10.2.920/x86/SafeExamBrowser.Client.exe") },
            PatchFile { name: "SafeExamBrowser.Configuration.dll", x64: include_bytes!("../../seb_resources/3.10.2.920/x64/SafeExamBrowser.Configuration.dll"), x86: include_bytes!("../../seb_resources/3.10.2.920/x86/SafeExamBrowser.Configuration.dll") },
            PatchFile { name: "SafeExamBrowser.Monitoring.dll", x64: include_bytes!("../../seb_resources/3.10.2.920/x64/SafeExamBrowser.Monitoring.dll"), x86: include_bytes!("../../seb_resources/3.10.2.920/x86/SafeExamBrowser.Monitoring.dll") },
            PatchFile { name: "SafeExamBrowser.Browser.dll", x64: include_bytes!("../../seb_resources/3.10.2.920/x64/SafeExamBrowser.Browser.dll"), x86: include_bytes!("../../seb_resources/3.10.2.920/x86/SafeExamBrowser.Browser.dll") },
        ],
    },
];

pub fn find_version(version: &str) -> Option<&'static SebVersion> {
    VERSIONS.iter().find(|v| v.version == version)
}

