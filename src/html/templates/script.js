        var currentContext = [];
        var currentIndex = 0;
        var slideshowInterval = null;

        function toggleView() {
            var flat = document.getElementById('flattened-gallery');
            var dirs = document.getElementById('directory-view');
            var btn = document.getElementById('toggle-btn');
            
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

        function openModal(element) {
            // Find the closest parent gallery container
            var gallery = element.closest('.gallery');
            if (!gallery) return;

            // Get all photo links in this specific gallery context
            var photos = gallery.querySelectorAll('.photo');
            currentContext = Array.from(photos).map(function(p) {
                var a = p.querySelector('a');
                var d = p.querySelector('.date');
                return {
                    src: a.getAttribute('href'),
                    date: d ? d.innerText : ''
                };
            });
            
            var targetHref = element.getAttribute('href');
            // Find index matching src
            for (var i = 0; i < currentContext.length; i++) {
                if (currentContext[i].src === targetHref) {
                    currentIndex = i;
                    break;
                }
            }
            
            if (currentIndex === -1) return; // Should not happen

            showModalImage();
            document.getElementById('modal').style.display = 'flex';
        }

        function closeModal() {
            document.getElementById('modal').style.display = 'none';
            stopSlideshow();
        }

        function showModalImage() {
            var img = document.getElementById('modal-img');
            var dateDiv = document.getElementById('modal-date');
            var item = currentContext[currentIndex];
            img.src = item.src;
            dateDiv.innerText = item.date;
            updateButtons();
        }

        function updateButtons() {
            // Disable Prev if at start
            document.getElementById('prev-btn').disabled = (currentIndex === 0);
            
            // Disable Next if at end (unless slideshow logic overrides, but standard button is for manual nav)
            // Requirement: "no de opcion hacia delante cuando es la ultima"
            document.getElementById('next-btn').disabled = (currentIndex === currentContext.length - 1);
        }

        function prevPhoto() {
            if (currentIndex > 0) {
                currentIndex--;
                showModalImage();
            }
        }

        function nextPhoto() {
            if (currentIndex < currentContext.length - 1) {
                currentIndex++;
                showModalImage();
            } else if (slideshowInterval) {
                // If slideshow is running and we reached the end, loop back to start
                currentIndex = 0;
                showModalImage();
            }
        }

        function toggleSlideshow() {
            if (slideshowInterval) {
                stopSlideshow();
            } else {
                startSlideshow();
            }
        }

        function startSlideshow() {
            var btn = document.getElementById('slideshow-btn');
            btn.innerText = 'Stop Slideshow';
            // Start immediately? or wait 10s? Usually wait.
            // If we are at the end, jump to start immediately?
            if (currentIndex === currentContext.length - 1) {
                currentIndex = 0;
                showModalImage();
            }
            slideshowInterval = setInterval(nextPhoto, 5000);
        }

        function stopSlideshow() {
            var btn = document.getElementById('slideshow-btn');
            if (btn) btn.innerText = 'Start Slideshow';
            if (slideshowInterval) {
                clearInterval(slideshowInterval);
                slideshowInterval = null;
            }
        }

        // Close modal when clicking outside image
        window.onclick = function(event) {
            var modal = document.getElementById('modal');
            if (event.target == modal) {
                closeModal();
            }
        }
        
        // Keyboard navigation
        document.onkeydown = function(e) {
            if (document.getElementById('modal').style.display === 'flex') {
                if (e.key === 'ArrowLeft') prevPhoto();
                if (e.key === 'ArrowRight') nextPhoto();
                if (e.key === 'Escape') closeModal();
            }
        }
