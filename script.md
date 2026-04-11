# BENTO Talk Script

## Introduction <Slide: Title Card>

* Who am I
* What do I do

## What is DJing <Slide: What is DJing>

1. "DJing is the art of playing multiple tracks and mixing them together to create a unique listening experience"
2. What do we want to DJ a track 
	* The basics:
		1. A way to play multiple tracks
    	2. A way to move the tracks
    	3. Apply effects on tracks
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

## Analyzing Tracks <Slide: none>

1. How do we create this data 
	* Many analysis programs exist
	* rekordbox
2. How is the data stored, loaded, and saved
	* DeviceSQL for storage, designed originally for the FitBit, reused by Pioneer DJ
	* Can perform SQL queries if you have a full DeviceSQL client

## Processing

1. The Dumb Method: Playing samples faster or slower (normal tempo) <Slide: none>
2. The Phase Vocoder <Slide: DFT and Spectrogram>
3. Playing continuous fft samples (master tempo) <Slide: DFT and Spectrogram>
4. Master Tempo vs Normal Tempo <Slide: none>

## Effects <Slide: Live Spectrogram>

1. EQ/Isolator Knobs {spectrogram animation: eq}
2. Filter Knobs {spectrogram animation: filter}
3. Reverb/Echo/Delay {spectrogram animation: delay}
