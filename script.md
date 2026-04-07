# BENTO Talk Script

## Introduction <Slide: Title Card>

* Who am I
* What do I do

## What is DJing <Slide: What is DJing>

1. "DJing is the art of playing multiple tracks and mixing them together to create a unique listening experience"
2. What do we want to DJ a track 
	* The basics:
		1. A way to play multiple tracks
    	2. A way to jog the tracks
	* Information we want: 
    	1. A way to see ahead
    	2. A way to go to a position
    	3. A way to sync BPM or Beats

## History of DJing <Slide: History of DJing>

1. Vinyl
	* Watching the Platter & Reading the Waveform {turntable animation: reading}
    * Cue Tape {turntable animation: positioning}
    * Aligning by Ear {turntable animation: aligning}
2. CDs
    * Small Waveform {cdj animation: reading}
    * Memory Cues {cdj animation: positioning}
    * BPM Readout {cdj animation: aligning}

## Modern DJing <Slide: none>

1. The Software
  	* Big Waveforms
  	* Hot Cues
  	* BPM and Beat Sync

## Analyzing Tracks <Slide: rekordbox>

1. How do we create this data 
	* Many analysis programs exist
	* Instead of reinventing the wheel, I used rekordbox
2. How is the data stored, loaded, and saved
	* DeviceSQL for storage, designed originally for the FitBit, reused by Pioneer DJ
	* Can perform SQL queries if you have a full DeviceSQL client
	* I use a dumber method: a library that directly reads out the table data and parses the binary format

## Processing

1. The Dumb Method: Playing samples faster or slower (normal tempo) <Slide: none>
2. The Fourier Transform <Slide: DFT and Spectrogram>
3. Playing continuous fft samples (master tempo) <Slide: DFT and Spectrogram>
4. Master Tempo vs Normal Tempo <Slide: none>

## Effects <Slide: Live Spectrogram>

2. EQ/Isolator Knobs {spectrogram animation: eq}
3. Filter Knobs {spectrogram animation: filter}
4. Effects
 	1. Reverb {spectrogram animation: reverb}
 	2. Echo {spectrogram animation: echo}
 	3. Transgate {spectrogram animation: transgate}
 	4. Roll {spectrogram animation: roll}
