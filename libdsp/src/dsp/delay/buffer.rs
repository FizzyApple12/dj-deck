/*
    /** Single-channel delay buffer

        Access is used with `buffer[]`, relative to the internal read/write position ("head").  This head is moved using `++buffer` (or `buffer += n`), such that `buffer[1] == (buffer + 1)[0]` in a similar way iterators/pointers.

        Operations like `buffer - 10` or `buffer++` return a View, which holds a fixed position in the buffer (based on the read/write position at the time).

        The capacity includes both positive and negative indices.  For example, a capacity of 100 would support using any of the ranges:

        * `buffer[-99]` to buffer[0]`
        * `buffer[-50]` to buffer[49]`
        * `buffer[0]` to buffer[99]`

        Although buffers are usually used with historical samples accessed using negative indices e.g. `buffer[-10]`, you could equally use it flipped around (moving the head backwards through the buffer using `--buffer`).
    */
    template<typename Sample>
    class Buffer {
        unsigned bufferIndex;
        unsigned bufferMask;
        std::vector<Sample> buffer;
    public:
        Buffer(int minCapacity=0) {
            resize(minCapacity);
        }
        // We shouldn't accidentally copy a delay buffer
        Buffer(const Buffer &other) = delete;
        Buffer & operator =(const Buffer &other) = delete;
        // But moving one is fine
        Buffer(Buffer &&other) = default;
        Buffer & operator =(Buffer &&other) = default;

        void resize(int minCapacity, Sample value=Sample()) {
            int bufferLength = 1;
            while (bufferLength < minCapacity) bufferLength *= 2;
            buffer.assign(bufferLength, value);
            bufferMask = unsigned(bufferLength - 1);
            bufferIndex = 0;
        }
        void reset(Sample value=Sample()) {
            buffer.assign(buffer.size(), value);
        }

        /// Holds a view for a particular position in the buffer
        template<bool isConst>
        class View {
            using CBuffer = typename std::conditional<isConst, const Buffer, Buffer>::type;
            using CSample = typename std::conditional<isConst, const Sample, Sample>::type;
            CBuffer *buffer = nullptr;
            unsigned bufferIndex = 0;
        public:
            View(CBuffer &buffer, int offset=0) : buffer(&buffer), bufferIndex(buffer.bufferIndex + (unsigned)offset) {}
            View(const View &other, int offset=0) : buffer(other.buffer), bufferIndex(other.bufferIndex + (unsigned)offset) {}
            View & operator =(const View &other) {
                buffer = other.buffer;
                bufferIndex = other.bufferIndex;
                return *this;
            }

            CSample & operator[](int offset) {
                return buffer->buffer[(bufferIndex + (unsigned)offset)&buffer->bufferMask];
            }
            const Sample & operator[](int offset) const {
                return buffer->buffer[(bufferIndex + (unsigned)offset)&buffer->bufferMask];
            }

            /// Write data into the buffer
            template<typename Data>
            void write(Data &&data, int length) {
                for (int i = 0; i < length; ++i) {
                    (*this)[i] = data[i];
                }
            }
            /// Read data out from the buffer
            template<typename Data>
            void read(int length, Data &&data) const {
                for (int i = 0; i < length; ++i) {
                    data[i] = (*this)[i];
                }
            }

            View operator +(int offset) const {
                return View(*this, offset);
            }
            View operator -(int offset) const {
                return View(*this, -offset);
            }
        };
        using MutableView = View<false>;
        using ConstView = View<true>;

        MutableView view(int offset=0) {
            return MutableView(*this, offset);
        }
        ConstView view(int offset=0) const {
            return ConstView(*this, offset);
        }
        ConstView constView(int offset=0) const {
            return ConstView(*this, offset);
        }

        Sample & operator[](int offset) {
            return buffer[(bufferIndex + (unsigned)offset)&bufferMask];
        }
        const Sample & operator[](int offset) const {
            return buffer[(bufferIndex + (unsigned)offset)&bufferMask];
        }

        /// Write data into the buffer
        template<typename Data>
        void write(Data &&data, int length) {
            for (int i = 0; i < length; ++i) {
                (*this)[i] = data[i];
            }
        }
        /// Read data out from the buffer
        template<typename Data>
        void read(int length, Data &&data) const {
            for (int i = 0; i < length; ++i) {
                data[i] = (*this)[i];
            }
        }

        Buffer & operator ++() {
            ++bufferIndex;
            return *this;
        }
        Buffer & operator +=(int i) {
            bufferIndex += (unsigned)i;
            return *this;
        }
        Buffer & operator --() {
            --bufferIndex;
            return *this;
        }
        Buffer & operator -=(int i) {
            bufferIndex -= (unsigned)i;
            return *this;
        }

        MutableView operator ++(int) {
            MutableView view(*this);
            ++bufferIndex;
            return view;
        }
        MutableView operator +(int i) {
            return MutableView(*this, i);
        }
        ConstView operator +(int i) const {
            return ConstView(*this, i);
        }
        MutableView operator --(int) {
            MutableView view(*this);
            --bufferIndex;
            return view;
        }
        MutableView operator -(int i) {
            return MutableView(*this, -i);
        }
        ConstView operator -(int i) const {
            return ConstView(*this, -i);
        }
    };
*/
