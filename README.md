# Useful links

https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/exports.html
https://djl-analysis.deepsymmetry.org/djl-analysis/packets.html

consider pruning:
* godot-ui
* godot-ui-code
* libui

rc1 todos:
* rewrite the ui with qt
* control quantize system
* usb-stored settings system
* finish button bindings
* rearchitect the audio pipeline
	* arena, nesting, proc macro
* build a custom Phase Vocoder around RustFFT
* fill in filter coefficients based on https://vicanek.de/articles/BiquadFits.pdf
* effects:
	* low cut echo
	* echo
	* delay (dub echo)
	* spiral
	* reverb
	* transgate
	* enigma jet
	* flanger
	* phaser
	* stretch
	* slip roll
	* roll
	* ping pong
	* helix
	* filter
	* mobius square
	* mobius saw
	* mobius triangle
	* noise
	* crush
	* pitch
* usb recording
* onelibrary support + deduplication with old rekordbox format
* usb library switching (for multiple libraries)
* switch waveform generation to use an onboard stft for consistency
* add graceful error handling for everything
* screen io board firmware
* mixer io board firmware
* player io board firmware

1.0 todos:
* pro dj link
* engine dj library support
* onboarding flow
* track analysis algorithm
* custom library format for onboard analysis
* track streaming support
* stems support
* stems splitter
* make final deckos distribution
