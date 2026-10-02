#pragma once

#include <QtCore/QMutex>
#include <QtCore/QObject>
#include <QtGui/QImage>
#include <QtQuick/QQuickItem>
#include <QtQuick/QSGTextureProvider>

struct GpuTextureSourceProvider : QSGTextureProvider {
    QSGTexture *texture() const override {
    	return this->provider_texture.get();
    }

    void setTexture(QSGTexture *new_texture) {
    	this->provider_texture.reset(new_texture);

     	emit textureChanged();
    }

    std::unique_ptr<QSGTexture> provider_texture;
};

class GpuTextureSource : public QQuickItem {
    Q_OBJECT
    QML_ELEMENT
public:
    GpuTextureSource() {
    	setFlag(ItemHasContents, true);
    }

    Q_INVOKABLE void setImage(const QImage &new_image) {
        {
        	QMutexLocker image_mutex_lock(&this->image_mutex);

         	this->pending_image = new_image;

          	this->image_updated = true;
        }

        update();
    }

    bool isTextureProvider() const override {
    	return true;
    }

    QSGTextureProvider *textureProvider() const override {
        auto *current_window = window();

        if (!current_window || !current_window->isSceneGraphInitialized()) {
        	return nullptr;
        }

        if (!this->provider) {
        	this->provider = new GpuTextureSourceProvider;
        }

        return this->provider;
    }

protected:
    QSGNode *updatePaintNode(QSGNode *paint_node, UpdatePaintNodeData *) override {
        QImage new_image;
        bool image_was_updated;

        {
        	QMutexLocker image_mutex_lock(&this->image_mutex);

         	new_image = std::move(this->pending_image);

          	image_was_updated = this->image_updated;
           	this->image_updated = false;
        }

        if (!this->provider) {
        	this->provider = new GpuTextureSourceProvider;
        }

        if (image_was_updated && this->provider) {
            QSGTexture *new_image_texture = window()->createTextureFromImage(
                new_image.convertedTo(QImage::Format_RGBA8888_Premultiplied),
                QQuickWindow::TextureHasAlphaChannel
            );

            new_image_texture->setFiltering(QSGTexture::Nearest);
            new_image_texture->setMipmapFiltering(QSGTexture::Nearest);
            new_image_texture->setHorizontalWrapMode(QSGTexture::ClampToEdge);
            new_image_texture->setVerticalWrapMode(QSGTexture::ClampToEdge);

            this->provider->setTexture(new_image_texture);
        }

        return paint_node;
    }

private:
    mutable GpuTextureSourceProvider *provider = nullptr;

    QMutex image_mutex;
    QImage pending_image;
    bool image_updated = false;
};
