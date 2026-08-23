use std::{ffi::c_void, ptr};

use core_foundation::{
    array::{CFArray, CFArrayRef},
    base::{CFIndex, CFType, CFTypeRef, TCFType},
    boolean::CFBoolean,
    dictionary::{CFDictionary, CFDictionaryRef},
    number::CFNumber,
    string::{CFString, CFStringRef},
    ConcreteCFType,
};
use core_graphics::{
    display::CGDisplay,
    geometry::CGRect,
    window::{
        kCGNullWindowID, kCGWindowBounds, kCGWindowLayer, kCGWindowListExcludeDesktopElements,
        kCGWindowListOptionOnScreenOnly, kCGWindowName, kCGWindowNumber, kCGWindowOwnerPID,
        CGWindowListCopyWindowInfo,
    },
};
use objc2_app_kit::{NSRunningApplication, NSWorkspace};

use crate::domain::foreground::{
    ForegroundReadError, ForegroundWindowSource, WindowMinimizer, WindowSnapshot,
};

type AXUIElementRef = *const c_void;
type AXError = i32;
type AXValueRef = *const c_void;

const AX_VALUE_CG_POINT: i32 = 1;
const AX_VALUE_CG_SIZE: i32 = 2;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> u8;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> u8;
    fn AXUIElementCreateApplication(process_id: libc::pid_t) -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> AXError;
    fn AXUIElementSetAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: CFTypeRef,
    ) -> AXError;
    fn AXValueGetValue(value: AXValueRef, value_type: i32, output: *mut c_void) -> u8;
    fn CFArrayGetCount(array: CFArrayRef) -> CFIndex;
    fn CFArrayGetValueAtIndex(array: CFArrayRef, index: CFIndex) -> *const c_void;
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
}

#[derive(Default)]
pub struct PlatformForegroundWindowSource;

#[derive(Default)]
pub struct PlatformWindowMinimizer;

impl PlatformForegroundWindowSource {
    pub const fn new() -> Self {
        Self
    }
}

pub fn focus_guard_permissions(request: bool) -> (bool, bool) {
    if request && unsafe { AXIsProcessTrusted() } == 0 {
        let key = CFString::from_static_string("AXTrustedCheckOptionPrompt");
        let prompt = CFBoolean::true_value();
        let options = CFDictionary::from_CFType_pairs(&[(key, prompt)]);
        unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) };
    }
    if request && !unsafe { CGPreflightScreenCaptureAccess() } {
        unsafe { CGRequestScreenCaptureAccess() };
    }
    (unsafe { AXIsProcessTrusted() } != 0, unsafe {
        CGPreflightScreenCaptureAccess()
    })
}

impl ForegroundWindowSource for PlatformForegroundWindowSource {
    fn foreground_window(&self) -> Result<Option<WindowSnapshot>, ForegroundReadError> {
        let workspace = NSWorkspace::sharedWorkspace();
        let Some(application) = workspace.frontmostApplication() else {
            return Ok(None);
        };
        let process_id = application.processIdentifier();
        if process_id <= 0 {
            return Ok(None);
        }
        let process_name = application.localizedName().map(|name| name.to_string());

        let windows =
            window_list(kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements)?;
        Ok(snapshot_for_process(&windows, process_id, process_name))
    }

    fn foreground_window_excluding(
        &self,
        excluded_process_id: u32,
    ) -> Result<Option<WindowSnapshot>, ForegroundReadError> {
        let workspace = NSWorkspace::sharedWorkspace();
        let Some(application) = workspace.frontmostApplication() else {
            return Ok(None);
        };
        let frontmost_process_id = application.processIdentifier();
        if frontmost_process_id <= 0 {
            return Ok(None);
        }
        if u32::try_from(frontmost_process_id).ok() != Some(excluded_process_id) {
            return self.foreground_window();
        }

        let windows =
            window_list(kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements)?;
        let Some(process_id) = windows.iter().find_map(|window| {
            let owner_pid = number_value(&window, unsafe { kCGWindowOwnerPID })?;
            let layer = number_value(&window, unsafe { kCGWindowLayer })?;
            if owner_pid == i64::from(excluded_process_id) || layer != 0 {
                return None;
            }
            let bounds = dictionary_value(&window, unsafe { kCGWindowBounds })?;
            let bounds = CGRect::from_dict_representation(&bounds)?;
            if bounds.size.width <= 1.0 || bounds.size.height <= 1.0 {
                return None;
            }
            libc::pid_t::try_from(owner_pid).ok()
        }) else {
            return Ok(None);
        };
        let process_name =
            NSRunningApplication::runningApplicationWithProcessIdentifier(process_id)
                .and_then(|application| application.localizedName().map(|name| name.to_string()));
        Ok(snapshot_for_process(&windows, process_id, process_name))
    }
}

fn snapshot_for_process(
    windows: &CFArray<CFDictionary<CFString, CFType>>,
    process_id: libc::pid_t,
    process_name: Option<String>,
) -> Option<WindowSnapshot> {
    windows.iter().find_map(|window| {
        let owner_pid = number_value(&window, unsafe { kCGWindowOwnerPID })?;
        let layer = number_value(&window, unsafe { kCGWindowLayer })?;
        if owner_pid != i64::from(process_id) || layer != 0 {
            return None;
        }

        let window_id = number_value(&window, unsafe { kCGWindowNumber })?;
        let bounds = dictionary_value(&window, unsafe { kCGWindowBounds })?;
        let bounds = CGRect::from_dict_representation(&bounds)?;
        if bounds.size.width <= 1.0 || bounds.size.height <= 1.0 {
            return None;
        }
        let display_bounds = display_containing(&bounds);
        let is_fullscreen = display_bounds.is_some_and(|display| {
            nearly_equal(bounds.origin.x, display.origin.x)
                && nearly_equal(bounds.origin.y, display.origin.y)
                && nearly_equal(bounds.size.width, display.size.width)
                && nearly_equal(bounds.size.height, display.size.height)
        });

        Some(WindowSnapshot {
            window_id: isize::try_from(window_id).ok()?,
            process_id: u32::try_from(process_id).ok()?,
            process_name: process_name.clone(),
            title: string_value(&window, unsafe { kCGWindowName }),
            is_visible: true,
            is_minimized: false,
            is_fullscreen,
            monitor_left: display_bounds
                .map_or(bounds.origin.x, |display| display.origin.x)
                .round() as i32,
            x: bounds.origin.x.round() as i32,
            y: bounds.origin.y.round() as i32,
            width: bounds.size.width.round().max(0.0) as u32,
            height: bounds.size.height.round().max(0.0) as u32,
        })
    })
}

impl WindowMinimizer for PlatformWindowMinimizer {
    fn minimize(&self, window_id: isize) -> Result<(), ForegroundReadError> {
        if unsafe { AXIsProcessTrusted() } == 0 {
            return Err(ForegroundReadError::AccessDenied);
        }
        let target = target_for_window(window_id)?;
        let application = unsafe { AXUIElementCreateApplication(target.process_id) };
        if application.is_null() {
            return Err(ForegroundReadError::InspectionFailed);
        }

        let focused_attribute = CFString::from_static_string("AXFocusedWindow");
        let mut focused_window: CFTypeRef = ptr::null();
        let copied = unsafe {
            AXUIElementCopyAttributeValue(
                application,
                focused_attribute.as_concrete_TypeRef(),
                &mut focused_window,
            )
        };
        if copied == 0 && !focused_window.is_null() {
            let matches = focused_window_matches(focused_window.cast(), &target.bounds);
            let result = matches.then(|| set_ax_minimized(focused_window.cast()));
            unsafe { core_foundation::base::CFRelease(focused_window) };
            if let Some(result) = result {
                unsafe { core_foundation::base::CFRelease(application.cast()) };
                return result;
            }
        }

        let windows_attribute = CFString::from_static_string("AXWindows");
        let mut windows: CFTypeRef = ptr::null();
        let copied = unsafe {
            AXUIElementCopyAttributeValue(
                application,
                windows_attribute.as_concrete_TypeRef(),
                &mut windows,
            )
        };
        if copied != 0 || windows.is_null() {
            unsafe { core_foundation::base::CFRelease(application.cast()) };
            return Err(ForegroundReadError::InspectionFailed);
        }
        let array = windows.cast();
        let count = unsafe { CFArrayGetCount(array) };
        let mut result = Err(ForegroundReadError::InspectionFailed);
        for index in 0..count {
            let window: AXUIElementRef = unsafe { CFArrayGetValueAtIndex(array, index) }.cast();
            if !window.is_null() && focused_window_matches(window, &target.bounds) {
                result = set_ax_minimized(window);
                break;
            }
        }
        unsafe {
            core_foundation::base::CFRelease(windows);
            core_foundation::base::CFRelease(application.cast());
        }
        result
    }
}

fn set_ax_minimized(window: AXUIElementRef) -> Result<(), ForegroundReadError> {
    let minimized_attribute = CFString::from_static_string("AXMinimized");
    let minimized = CFBoolean::true_value();
    let result = unsafe {
        AXUIElementSetAttributeValue(
            window,
            minimized_attribute.as_concrete_TypeRef(),
            minimized.as_CFTypeRef(),
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(ForegroundReadError::InspectionFailed)
    }
}

fn window_list(
    options: u32,
) -> Result<CFArray<CFDictionary<CFString, CFType>>, ForegroundReadError> {
    let raw = unsafe { CGWindowListCopyWindowInfo(options, kCGNullWindowID) };
    if raw.is_null() {
        Err(ForegroundReadError::InspectionFailed)
    } else {
        Ok(unsafe { CFArray::wrap_under_create_rule(raw) })
    }
}

fn typed_value<T: ConcreteCFType>(
    dictionary: &CFDictionary<CFString, CFType>,
    key: CFStringRef,
) -> Option<T> {
    let key = unsafe { CFString::wrap_under_get_rule(key) };
    dictionary.find(&key)?.downcast::<T>()
}

fn number_value(dictionary: &CFDictionary<CFString, CFType>, key: CFStringRef) -> Option<i64> {
    typed_value::<CFNumber>(dictionary, key)?.to_i64()
}

fn string_value(dictionary: &CFDictionary<CFString, CFType>, key: CFStringRef) -> Option<String> {
    typed_value::<CFString>(dictionary, key).map(|value| value.to_string())
}

fn dictionary_value(
    dictionary: &CFDictionary<CFString, CFType>,
    key: CFStringRef,
) -> Option<CFDictionary> {
    typed_value::<CFDictionary>(dictionary, key)
}

fn display_containing(window: &CGRect) -> Option<CGRect> {
    let center_x = window.origin.x + window.size.width / 2.0;
    let center_y = window.origin.y + window.size.height / 2.0;
    CGDisplay::active_displays()
        .ok()?
        .into_iter()
        .find_map(|id| {
            let bounds = CGDisplay::new(id).bounds();
            (center_x >= bounds.origin.x
                && center_x < bounds.origin.x + bounds.size.width
                && center_y >= bounds.origin.y
                && center_y < bounds.origin.y + bounds.size.height)
                .then_some(bounds)
        })
}

fn nearly_equal(left: f64, right: f64) -> bool {
    (left - right).abs() <= 8.0
}

struct WindowTarget {
    process_id: libc::pid_t,
    bounds: CGRect,
}

fn target_for_window(window_id: isize) -> Result<WindowTarget, ForegroundReadError> {
    let windows = window_list(0)?;
    windows
        .iter()
        .find_map(|window| {
            let candidate = number_value(&window, unsafe { kCGWindowNumber })?;
            if isize::try_from(candidate).ok()? != window_id {
                return None;
            }
            let process_id = number_value(&window, unsafe { kCGWindowOwnerPID })?;
            let bounds = dictionary_value(&window, unsafe { kCGWindowBounds })?;
            Some(WindowTarget {
                process_id: libc::pid_t::try_from(process_id).ok()?,
                bounds: CGRect::from_dict_representation(&bounds)?,
            })
        })
        .ok_or(ForegroundReadError::InspectionFailed)
}

fn focused_window_matches(window: AXUIElementRef, target: &CGRect) -> bool {
    let Some(position) =
        copy_ax_value::<core_graphics::geometry::CGPoint>(window, "AXPosition", AX_VALUE_CG_POINT)
    else {
        return false;
    };
    let Some(size) =
        copy_ax_value::<core_graphics::geometry::CGSize>(window, "AXSize", AX_VALUE_CG_SIZE)
    else {
        return false;
    };
    geometry_value_matches(position.x, target.origin.x)
        && geometry_value_matches(position.y, target.origin.y)
        && geometry_value_matches(size.width, target.size.width)
        && geometry_value_matches(size.height, target.size.height)
}

fn copy_ax_value<T: Default>(
    element: AXUIElementRef,
    attribute: &'static str,
    value_type: i32,
) -> Option<T> {
    let attribute = CFString::from_static_string(attribute);
    let mut value: CFTypeRef = ptr::null();
    let copied = unsafe {
        AXUIElementCopyAttributeValue(element, attribute.as_concrete_TypeRef(), &mut value)
    };
    if copied != 0 || value.is_null() {
        return None;
    }
    let mut output = T::default();
    let succeeded =
        unsafe { AXValueGetValue(value.cast(), value_type, (&mut output as *mut T).cast()) };
    unsafe { core_foundation::base::CFRelease(value) };
    (succeeded != 0).then_some(output)
}

fn geometry_value_matches(left: f64, right: f64) -> bool {
    (left - right).abs() <= 8.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreground_snapshot_is_safe_to_query() {
        let result = PlatformForegroundWindowSource::new().foreground_window();
        assert!(result.is_ok());
    }

    #[test]
    fn fullscreen_comparison_allows_small_frame_differences() {
        assert!(nearly_equal(100.0, 107.9));
        assert!(!nearly_equal(100.0, 108.1));
    }

    #[test]
    fn intervention_geometry_requires_a_tight_match() {
        assert!(geometry_value_matches(100.0, 107.9));
        assert!(!geometry_value_matches(100.0, 108.1));
    }
}
