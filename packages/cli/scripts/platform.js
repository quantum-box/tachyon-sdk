"use strict";

function resolvePlatformTarget(platform = process.platform, arch = process.arch) {
  const osByPlatform = {
    darwin: "darwin",
    linux: "linux",
  };
  const archByNodeArch = {
    arm64: "arm64",
    x64: "x86_64",
  };

  const os = osByPlatform[platform];
  const artifactArch = archByNodeArch[arch];

  if (!os || !artifactArch) {
    throw new Error(
      `Unsupported platform: ${platform}/${arch}. Tachyon CLI npm installs currently support macOS and Linux on arm64 or x64.`,
    );
  }

  return {
    os,
    arch: artifactArch,
    artifactName: `tachyon-${os}-${artifactArch}`,
  };
}

module.exports = {
  resolvePlatformTarget,
};
