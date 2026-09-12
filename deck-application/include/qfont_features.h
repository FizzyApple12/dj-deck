#pragma once
#include <QFont>
#include <QString>

inline void set_qfont_feature(QFont &font, const QString &tag, quint32 value) {
    std::optional<QFont::Tag> raw_tag = QFont::Tag::fromString(tag);

    if (raw_tag.has_value()) {
        font.setFeature(*raw_tag, value);
    }
}
