#import "Temporal.h"
#import "temporal_rn.h"

@implementation Temporal

- (NSNumber *)multiply:(double)a b:(double)b {
    NSNumber *result = @(a * b);
    return result;
}

- (NSString *)instantNow {
    char *result = temporal_instant_now();
    if (result == NULL) {
        return @"";
    }
    NSString *nsResult = [NSString stringWithUTF8String:result];
    temporal_free_string(result);
    return nsResult;
}

- (std::shared_ptr<facebook::react::TurboModule>)getTurboModule:
    (const facebook::react::ObjCTurboModule::InitParams &)params
{
    return std::make_shared<facebook::react::NativeTemporalSpecJSI>(params);
}

+ (NSString *)moduleName
{
    return @"Temporal";
}

@end
