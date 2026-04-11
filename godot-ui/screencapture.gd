extends TextureRect

func _ready() -> void:
	var capture := DesktopCaptureTexture.new()
	capture.monitor_index = 0     # primary monitor
	capture.capture_cursor = true
	capture.max_fps = 60

	texture = capture

	capture.capture_stopped.connect(_on_capture_stopped)
	#capture.capture_stats_updated.connect(func(stats: Dictionary) -> void:
		#print("capture fps=", stats.get("estimated_capture_fps", 0.0), " late=", stats.get("late_frame_count", 0))
	#)
	#capture.diagnostics_enabled = true
	capture.enabled = true

func _on_capture_stopped(reason: String) -> void:
	push_warning("Capture stopped: " + reason)
