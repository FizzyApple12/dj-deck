pub mod interpolators;

use std::marker::PhantomData;

use num::Float;

/*
    /** @brief A delay-line reader which uses an external buffer

        This is useful if you have multiple delay-lines reading from the same buffer.
    */
    template<class Sample, template<typename> class Interpolator=InterpolatorLinear>
    class Reader : public Interpolator<Sample> /* so we can get the empty-base-class optimisation */ {
        using Super = Interpolator<Sample>;
    public:
        Reader () {}
        /// Pass in a configured interpolator
        Reader (const Interpolator<Sample> &interpolator) : Super(interpolator) {}

        template<typename Buffer>
        Sample read(const Buffer &buffer, Sample delaySamples) const {
            int startIndex = delaySamples;
            Sample remainder = delaySamples - startIndex;

            // Delay buffers use negative indices, but interpolators use positive ones
            using View = decltype(buffer - startIndex);
            struct Flipped {
                 View view;
                 Sample operator [](int i) const {
                    return view[-i];
                 }
            };
            return Super::fractional(Flipped{buffer - startIndex}, remainder);
        }
    };

    /**	@brief A single-channel delay-line containing its own buffer.*/
    template<class Sample, template<typename> class Interpolator=InterpolatorLinear>
    class Delay : private Reader<Sample, Interpolator> {
        using Super = Reader<Sample, Interpolator>;
        Buffer<Sample> buffer;
    public:
        static constexpr Sample latency = Super::latency;

        Delay(int capacity=0) : buffer(1 + capacity + Super::inputLength) {}
        /// Pass in a configured interpolator
        Delay(const Interpolator<Sample> &interp, int capacity=0) : Super(interp), buffer(1 + capacity + Super::inputLength) {}

        void reset(Sample value=Sample()) {
            buffer.reset(value);
        }
        void resize(int minCapacity, Sample value=Sample()) {
            buffer.resize(minCapacity + Super::inputLength, value);
        }

        /** Read a sample from `delaySamples` >= 0 in the past.
        The interpolator may add its own latency on top of this (see `Delay::latency`).  The default interpolation (linear) has 0 latency.
        */
        Sample read(Sample delaySamples) const {
            return Super::read(buffer, delaySamples);
        }
        /// Writes a sample. Returns the same object, so that you can say `delay.write(v).read(delay)`.
        Delay & write(Sample value) {
            ++buffer;
            buffer[0] = value;
            return *this;
        }
    };

    /**	@brief A multi-channel delay-line with its own buffer. */
    template<class Sample, template<typename> class Interpolator=InterpolatorLinear>
    class MultiDelay : private Reader<Sample, Interpolator> {
        using Super = Reader<Sample, Interpolator>;
        int channels;
        MultiBuffer<Sample> multiBuffer;
    public:
        static constexpr Sample latency = Super::latency;

        MultiDelay(int channels=0, int capacity=0) : channels(channels), multiBuffer(channels, 1 + capacity + Super::inputLength) {}

        void reset(Sample value=Sample()) {
            multiBuffer.reset(value);
        }
        void resize(int nChannels, int capacity, Sample value=Sample()) {
            channels = nChannels;
            multiBuffer.resize(channels, capacity + Super::inputLength, value);
        }

        /// A single-channel delay-line view, similar to a `const Delay`
        struct ChannelView {
            static constexpr Sample latency = Super::latency;

            const Super &reader;
            typename MultiBuffer<Sample>::ConstChannel channel;

            Sample read(Sample delaySamples) const {
                return reader.read(channel, delaySamples);
            }
        };
        ChannelView operator [](int channel) const {
            return ChannelView{*this, multiBuffer[channel]};
        }

        /// A multi-channel result, lazily calculating samples
        struct DelayView {
            Super &reader;
            typename MultiBuffer<Sample>::ConstView view;
            Sample delaySamples;

            // Calculate samples on-the-fly
            Sample operator [](int c) const {
                return reader.read(view[c], delaySamples);
            }
        };
        DelayView read(Sample delaySamples) {
            return DelayView{*this, multiBuffer.constView(), delaySamples};
        }
        /// Reads into the provided output structure
        template<class Output>
        void read(Sample delaySamples, Output &output) {
            for (int c = 0; c < channels; ++c) {
                output[c] = Super::read(multiBuffer[c], delaySamples);
            }
        }
        /// Reads separate delays for each channel
        template<class Delays, class Output>
        void readMulti(const Delays &delays, Output &output) {
            for (int c = 0; c < channels; ++c) {
                output[c] = Super::read(multiBuffer[c], delays[c]);
            }
        }
        template<class Data>
        MultiDelay & write(const Data &data) {
            ++multiBuffer;
            for (int c = 0; c < channels; ++c) {
                multiBuffer[c][0] = data[c];
            }
            return *this;
        }
    };

*/
