require "json"

package = JSON.parse(File.read(File.join(__dir__, "package.json")))

Pod::Spec.new do |s|
  s.name         = "Temporal"
  s.version      = package["version"]
  s.summary      = package["description"]
  s.homepage     = package["homepage"]
  s.license      = package["license"]
  s.authors      = package["author"]

  s.platforms    = { :ios => min_ios_version_supported }
  s.source       = { :git => "https://github.com/V3RON/react-native-temporal.git", :tag => "#{s.version}" }

  s.source_files = "ios/**/*.{h,m,mm,swift,cpp}"
  s.private_header_files = "ios/**/*.h"

  # Rust static library configuration
  s.preserve_paths = "ios/libs/*.a", "ios/temporal_rn.h"

  s.pod_target_xcconfig = {
    'HEADER_SEARCH_PATHS' => '"$(PODS_TARGET_SRCROOT)/ios"',
    'LIBRARY_SEARCH_PATHS' => '"$(PODS_TARGET_SRCROOT)/ios/libs"',
    'OTHER_LDFLAGS[sdk=iphoneos*]' => '-ltemporal_rn_device',
    'OTHER_LDFLAGS[sdk=iphonesimulator*]' => '-ltemporal_rn_sim'
  }

  install_modules_dependencies(s)
end
