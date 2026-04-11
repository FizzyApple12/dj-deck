extends Node

var slide = 0

var blur_show_animation;

var turntable_animation;
var cdj_animation;
var dft_animation;

var title_card;
var thank_you_card;
var history_of_djing;
var dft_and_spectrogram;

func _ready() -> void:
	blur_show_animation = get_node("%blur show") as AnimationPlayer
	
	turntable_animation = get_node("%turntable animation") as AnimationPlayer
	cdj_animation = get_node("%cdj animation") as AnimationPlayer
	dft_animation = get_node("%dft animation") as AnimationPlayer

	title_card = get_node("%Title Card") as CanvasItem
	thank_you_card = get_node("%Thank You Card") as CanvasItem
	history_of_djing = get_node("%History of DJing") as CanvasItem
	dft_and_spectrogram = get_node("%DFT and Spectrogram") as CanvasItem

func _process(_delta: float) -> void:
	match slide:
		0:
			title_card.set_visible(false)
			history_of_djing.set_visible(false)
			dft_and_spectrogram.set_visible(false)
			thank_you_card.set_visible(false)
			
		1:
			title_card.set_visible(true)
			history_of_djing.set_visible(false)
			dft_and_spectrogram.set_visible(false)
			thank_you_card.set_visible(false)
			
		2:
			title_card.set_visible(false)
			history_of_djing.set_visible(false)
			dft_and_spectrogram.set_visible(false)
			thank_you_card.set_visible(false)
			
		3:
			title_card.set_visible(false)
			history_of_djing.set_visible(true)
			dft_and_spectrogram.set_visible(false)
			thank_you_card.set_visible(false)
		4:
			title_card.set_visible(false)
			history_of_djing.set_visible(true)
			dft_and_spectrogram.set_visible(false)
			thank_you_card.set_visible(false)
		5:
			title_card.set_visible(false)
			history_of_djing.set_visible(true)
			dft_and_spectrogram.set_visible(false)
			thank_you_card.set_visible(false)
		6:
			title_card.set_visible(false)
			history_of_djing.set_visible(true)
			dft_and_spectrogram.set_visible(false)
			thank_you_card.set_visible(false)
			
		7:
			title_card.set_visible(false)
			history_of_djing.set_visible(true)
			dft_and_spectrogram.set_visible(false)
			thank_you_card.set_visible(false)
		8:
			title_card.set_visible(false)
			history_of_djing.set_visible(true)
			dft_and_spectrogram.set_visible(false)
			thank_you_card.set_visible(false)
		9:
			title_card.set_visible(false)
			history_of_djing.set_visible(true)
			dft_and_spectrogram.set_visible(false)
			thank_you_card.set_visible(false)
			
		10:
			title_card.set_visible(false)
			history_of_djing.set_visible(false)
			dft_and_spectrogram.set_visible(false)
			thank_you_card.set_visible(false)
			
		11:
			title_card.set_visible(false)
			history_of_djing.set_visible(false)
			dft_and_spectrogram.set_visible(true)
			thank_you_card.set_visible(false)
		12:
			title_card.set_visible(false)
			history_of_djing.set_visible(false)
			dft_and_spectrogram.set_visible(true)
			thank_you_card.set_visible(false)
		13:
			title_card.set_visible(false)
			history_of_djing.set_visible(false)
			dft_and_spectrogram.set_visible(true)
			thank_you_card.set_visible(false)
		14:
			title_card.set_visible(false)
			history_of_djing.set_visible(false)
			dft_and_spectrogram.set_visible(true)
			thank_you_card.set_visible(false)
		15:
			title_card.set_visible(false)
			history_of_djing.set_visible(false)
			dft_and_spectrogram.set_visible(true)
			thank_you_card.set_visible(false)
		16:
			title_card.set_visible(false)
			history_of_djing.set_visible(false)
			dft_and_spectrogram.set_visible(true)
			thank_you_card.set_visible(false)
		17:
			title_card.set_visible(false)
			history_of_djing.set_visible(false)
			dft_and_spectrogram.set_visible(true)
			thank_you_card.set_visible(false)
			
		18:
			title_card.set_visible(false)
			history_of_djing.set_visible(false)
			dft_and_spectrogram.set_visible(false)
			thank_you_card.set_visible(false)
			
		19:
			title_card.set_visible(false)
			history_of_djing.set_visible(false)
			dft_and_spectrogram.set_visible(false)
			thank_you_card.set_visible(true)
			
		_:
			pass
		

func _input(event):
	if event.is_action_pressed("slide_advance"):
		slide += 1;
		match slide:
			0:
				blur_show_animation.play("hide")
				
			1:
				blur_show_animation.play("show")
				
			2:
				blur_show_animation.play("hide")
				
			3:
				blur_show_animation.play("show")
				
			4:
				turntable_animation.play("reading")
			5:
				turntable_animation.play("positioning")
			6:
				turntable_animation.play("aligning")
				
			7:
				cdj_animation.play("reading")
			8:
				cdj_animation.play("positioning")
			9:
				cdj_animation.play("aligning")
				
			10:
				blur_show_animation.play("hide")
				
			11:
				blur_show_animation.play("show")
			12:
				dft_animation.play("dft")
			13:
				dft_animation.play("spectrogram")
			14:
				dft_animation.play("spectrogram-playback")
			15:
				dft_animation.play("eq")
			16:
				dft_animation.play("filter")
			17:
				dft_animation.play("delay")
				
			18:
				blur_show_animation.play("hide")
				
			_:
				pass

	if event.is_action_pressed("slide_back"):
		match slide:
			0:
				blur_show_animation.play_backwards("hide")
				
			1:
				blur_show_animation.play_backwards("show")
				
			2:
				blur_show_animation.play_backwards("hide")
				
			3:
				blur_show_animation.play_backwards("show")
				
			4:
				turntable_animation.play_backwards("reading")
			5:
				turntable_animation.play_backwards("positioning")
			6:
				turntable_animation.play_backwards("aligning")
				
			7:
				cdj_animation.play_backwards("reading")
			8:
				cdj_animation.play_backwards("positioning")
			9:
				cdj_animation.play_backwards("aligning")
				
			10:
				blur_show_animation.play_backwards("hide")
				
			11:
				blur_show_animation.play_backwards("show")
			12:
				dft_animation.play_backwards("dft")
			13:
				dft_animation.play_backwards("spectrogram")
			14:
				dft_animation.play_backwards("spectrogram-playback")
			15:
				dft_animation.play_backwards("eq")
			16:
				dft_animation.play_backwards("filter")
			17:
				dft_animation.play_backwards("delay")
				
			18:
				blur_show_animation.play_backwards("hide")
				
			_:
				pass
		slide -= 1;
