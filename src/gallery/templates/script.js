var currentContext = [];
        var currentIndex = 0;
        var slideshowInterval = null;

        function filterGallery(type, btn) {
            // Update active state of buttons
            document.querySelectorAll('.filter-btn').forEach(function(b) {
                b.classList.remove('active');
            });
            btn.classList.add('active');

            // Filter photos in both galleries
            document.querySelectorAll('.photo').forEach(function(p) {
                var pType = p.querySelector('a').getAttribute('data-type');
                if (type === 'all' || pType === type) {
                    p.style.display = 'block';
                } else {
                    p.style.display = 'none';
                }
            });
        }

        function toggleView() {
            var flat = document.getElementById('flattened-gallery');
            var dirs = document.getElementById('directory-view');
            var btn = document.getElementById('toggle-btn');
            
            if (flat && dirs && btn) {
                if (flat.style.display !== 'none') {
                    flat.style.display = 'none';
                    dirs.style.display = 'block';
                    btn.innerText = 'Show Photos';
                } else {
                    flat.style.display = 'grid';
                    dirs.style.display = 'none';
                    btn.innerText = 'Show Days';
                }
            }
        }

        function openModal(linkElement) {
            var gallery = linkElement.closest('.gallery');
            if (!gallery) return;

            var photos = gallery.querySelectorAll('.photo');
            currentContext = Array.from(photos).map(function(p) {
                var a = p.querySelector('a');
                var img = p.querySelector('img');
                var d = p.querySelector('.info-overlay');
                return {
                    src: a.getAttribute('href'),
                    displaySrc: img.getAttribute('src'),
                    type: a.getAttribute('data-type') || 'image',
                    date: d ? d.innerText : ''
                };
            });

            var targetHref = linkElement.getAttribute('href');
            currentIndex = currentContext.findIndex(function(item) {
                return item.src === targetHref;
            });

            if (currentIndex === -1) currentIndex = 0;

            updateModalContent();
            document.getElementById('modal').style.display = 'flex';
        }

        function closeModal(event) {
            if (event) event.stopPropagation();
            document.getElementById('modal').style.display = 'none';
            stopSlideshow();
            var vid = document.getElementById('modal-video');
            if (vid) {
                vid.pause();
                vid.removeAttribute('src'); // Fully clear source attribute
                vid.load();
            }
        }

        function updateModalContent() {
            var img = document.getElementById('modal-img');
            var vid = document.getElementById('modal-video');
            var dateDiv = document.getElementById('modal-date');
            var item = currentContext[currentIndex];

            dateDiv.innerText = item.date;

            // Pause video if it was playing
            vid.pause();

            if (item.type === 'video') {
                img.style.display = 'none';
                vid.style.display = 'block';
                vid.src = item.src;
                vid.load(); // Force the video to load the new source
                if (slideshowInterval) {
                    vid.play(); 
                }
            } else {
                img.style.display = 'block';
                vid.style.display = 'none';
                img.src = item.src; // Full image
            }
            
            updateButtons();
        }

        function updateButtons() {
            document.getElementById('prev-btn').disabled = (currentIndex === 0);
            document.getElementById('next-btn').disabled = (currentIndex === currentContext.length - 1);
        }

        function changeSlide(n) {
            var nextIndex = currentIndex + n;
            if (nextIndex >= 0 && nextIndex < currentContext.length) {
                currentIndex = nextIndex;
                updateModalContent();
            } else if (slideshowInterval && nextIndex >= currentContext.length) {
                // Loop in slideshow
                currentIndex = 0;
                updateModalContent();
            }
        }

        function toggleSlideshow() {
            if (slideshowInterval) {
                stopSlideshow();
                // Update content to reflect manual mode (show video player if current is video)
                updateModalContent(); 
            } else {
                startSlideshow();
            }
        }

        function startSlideshow() {
            var btn = document.getElementById('slideshow-btn');
            if (btn) btn.innerText = 'Stop Slideshow';
            
            // If starting slideshow on a video, switch to thumbnail immediately
            updateModalContent();

            slideshowInterval = setInterval(function() {
                changeSlide(1);
            }, 5000);
        }

        function stopSlideshow() {
            var btn = document.getElementById('slideshow-btn');
            if (btn) btn.innerText = 'Start Slideshow';
            if (slideshowInterval) {
                clearInterval(slideshowInterval);
                slideshowInterval = null;
            }
        }

        // Keyboard navigation
        document.onkeydown = function(e) {
            if (document.getElementById('modal').style.display === 'flex') {
                if (e.key === 'ArrowLeft') changeSlide(-1);
                if (e.key === 'ArrowRight') changeSlide(1);
                if (e.key === 'Escape') closeModal();
            }
        }
