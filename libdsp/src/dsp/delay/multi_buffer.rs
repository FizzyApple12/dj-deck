/*
/** @brief Multi-channel delay buffer

    This behaves similarly to the single-channel `Buffer`, with the following differences:

    * `buffer[c]` returns a view for a single channel, which behaves like the single-channel `Buffer::View`.
    * The constructor and `.resize()` take an additional first `channel` argument.
*/
template<typename Sample>
class MultiBuffer {
    int channels, stride;
    Buffer<Sample> buffer;
public:
    using ConstChannel = typename Buffer<Sample>::ConstView;
    using MutableChannel = typename Buffer<Sample>::MutableView;

    MultiBuffer(int channels=0, int capacity=0) : channels(channels), stride(capacity), buffer(channels*capacity) {}

    void resize(int nChannels, int capacity, Sample value=Sample()) {
        channels = nChannels;
        stride = capacity;
        buffer.resize(channels*capacity, value);
    }
    void reset(Sample value=Sample()) {
        buffer.reset(value);
    }

    /// A reference-like multi-channel result for a particular sample index
    template<bool isConst>
    class Stride {
        using CChannel = typename std::conditional<isConst, ConstChannel, MutableChannel>::type;
        using CSample = typename std::conditional<isConst, const Sample, Sample>::type;
        CChannel view;
        int channels, stride;
    public:
        Stride(CChannel view, int channels, int stride) : view(view), channels(channels), stride(stride) {}
        Stride(const Stride &other) : view(other.view), channels(other.channels), stride(other.stride) {}

        CSample & operator[](int channel) {
            return view[channel*stride];
        }
        const Sample & operator[](int channel) const {
            return view[channel*stride];
        }

        /// Reads from the buffer into a multi-channel result
        template<class Data>
        void get(Data &&result) const {
            for (int c = 0; c < channels; ++c) {
                result[c] = view[c*stride];
            }
        }
        /// Writes from multi-channel data into the buffer
        template<class Data>
        void set(Data &&data) {
            for (int c = 0; c < channels; ++c) {
                view[c*stride] = data[c];
            }
        }
        template<class Data>
        Stride & operator =(const Data &data) {
            set(data);
            return *this;
        }
        Stride & operator =(const Stride &data) {
            set(data);
            return *this;
        }
    };

    Stride<false> at(int offset) {
        return {buffer.view(offset), channels, stride};
    }
    Stride<true> at(int offset) const {
        return {buffer.view(offset), channels, stride};
    }

    /// Holds a particular position in the buffer
    template<bool isConst>
    class View {
        using CChannel = typename std::conditional<isConst, ConstChannel, MutableChannel>::type;
        CChannel view;
        int channels, stride;
    public:
        View(CChannel view, int channels, int stride) : view(view), channels(channels), stride(stride) {}

        CChannel operator[](int channel) {
            return view + channel*stride;
        }
        ConstChannel operator[](int channel) const {
            return view + channel*stride;
        }

        Stride<isConst> at(int offset) {
            return {view + offset, channels, stride};
        }
        Stride<true> at(int offset) const {
            return {view + offset, channels, stride};
        }
    };
    using MutableView = View<false>;
    using ConstView = View<true>;

    MutableView view(int offset=0) {
        return MutableView(buffer.view(offset), channels, stride);
    }
    ConstView view(int offset=0) const {
        return ConstView(buffer.view(offset), channels, stride);
    }
    ConstView constView(int offset=0) const {
        return ConstView(buffer.view(offset), channels, stride);
    }

    MutableChannel operator[](int channel) {
        return buffer + channel*stride;
    }
    ConstChannel operator[](int channel) const {
        return buffer + channel*stride;
    }

    MultiBuffer & operator ++() {
        ++buffer;
        return *this;
    }
    MultiBuffer & operator +=(int i) {
        buffer += i;
        return *this;
    }
    MutableView operator ++(int) {
        return MutableView(buffer++, channels, stride);
    }
    MutableView operator +(int i) {
        return MutableView(buffer + i, channels, stride);
    }
    ConstView operator +(int i) const {
        return ConstView(buffer + i, channels, stride);
    }
    MultiBuffer & operator --() {
        --buffer;
        return *this;
    }
    MultiBuffer & operator -=(int i) {
        buffer -= i;
        return *this;
    }
    MutableView operator --(int) {
        return MutableView(buffer--, channels, stride);
    }
    MutableView operator -(int i) {
        return MutableView(buffer - i, channels, stride);
    }
    ConstView operator -(int i) const {
        return ConstView(buffer - i, channels, stride);
    }
};
*/
