# Useful links

https://djl-analysis.deepsymmetry.org/rekordbox-export-analysis/exports.html
https://djl-analysis.deepsymmetry.org/djl-analysis/packets.html

rc1 todos:
* rewrite the ui with iced
* rearchitect the audio pipeline
	* arena, nesting, more?
	* standard effects architecture
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
* usb-stored settings system
* onelibrary support + deduplication with old rekordbox format
* engine dj library support
* usb library switching (for multiple libraries)
* pro dj link
* separate io logic and mapping logic and move mapping logic to libdj
* make generic control mapping system
* switch waveform generation to use an onboard stft for consistency
* add a global fault management system with graceful degradation and retry logic
* add graceful error handling for everything
* mixer io board firmware
* player io board firmware
* make final deckos distribution

1.0 todos:
* track analysis algorithm
* custom library format for onboard analysis
* track streaming support
* stems support
* stems splitter
